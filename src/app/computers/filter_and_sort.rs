use crate::{
    app::states::pane::settings::{RetentionTimes, Settings, Threshold},
    r#const::*,
    utils::hash::HashedDataFrame,
};
use const_format::formatcp;
use egui::util::cache::{ComputerMut, FrameCache};
use polars::prelude::*;
use scirs2::spatial::cosine;
use std::{f64, ops::Sub};
use tracing::{debug, error, instrument, trace};

/// Peak computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Peak computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    #[instrument(skip_all, err)]
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        lazy_frame = compute(lazy_frame, key)?;
        lazy_frame = filter_and_sort(lazy_frame, key);
        let data_frame = lazy_frame.collect()?;
        trace!(?data_frame);
        Ok(HashedDataFrame::new(data_frame)?)
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Peak key
#[derive(Clone, Copy, Hash, Debug)]
pub struct Key<'a> {
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) threshold: Threshold,
    pub(crate) retention_times: &'a RetentionTimes,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame,
            threshold: settings.threshold,
            retention_times: &settings.retention_times,
        }
    }
}

/// Peak value
type Value = HashedDataFrame;

/// Format
fn compute(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    // let index = (col(RETENTION_TIME) - lit(key.threshold.retention_time.0))
    //     .abs()
    //     .arg_min();
    // let index = (col(RETENTION_TIME) - lit(key.threshold.retention_time.0))
    //     .abs()
    //     .arg_sort(false, false);
    // let pivot = temp.pivot(
    //     cols([RETENTION_TIME]),
    //     on_columns,
    //     cols([MASS_TO_CHARGE]),
    //     cols([SIGNAL]),
    //     element(),
    //     false,
    //     PlSmallStr::from_static("_"),
    // );
    debug!(lazy_frame = %lazy_frame.clone().collect().unwrap());
    let retention_time = lazy_frame.clone().select([col(RETENTION_TIME)]);
    debug!(retention_time = %retention_time.clone().collect().unwrap());
    // В конце необходима группировка, так как иначе массы 99.5 и 100.4 после округления дадут две строки с 100.0
    let explode = lazy_frame
        .clone()
        .select([col(RETENTION_TIME), col(MASS_SPECTRUM)])
        .explode(
            cols([MASS_SPECTRUM]),
            ExplodeOptions {
                empty_as_null: false,
                keep_nulls: false,
            },
        )
        .unnest(cols([MASS_SPECTRUM]), None)
        .with_column(col(MASS_TO_CHARGE).round(0, RoundMode::HalfToEven))
        .group_by([col(RETENTION_TIME), col(MASS_TO_CHARGE)])
        .agg([col(SIGNAL).mean()]);
    debug!(explode = %explode.clone().collect().unwrap());
    let mass_to_charge = explode.clone().select([col(MASS_TO_CHARGE).unique()]);
    debug!(mass_to_charge = %mass_to_charge.clone().collect().unwrap());
    // Получаем все возможные пары (RETENTION_TIME, MASS_TO_CHARGE)
    let cross_join = retention_time.cross_join(mass_to_charge, None);
    debug!(cross_join = %cross_join.clone().collect().unwrap());
    // Сопоставляем с SIGNAL
    let join = cross_join.join(
        explode,
        [col(RETENTION_TIME), col(MASS_TO_CHARGE)],
        [col(RETENTION_TIME), col(MASS_TO_CHARGE)],
        JoinArgs::new(JoinType::Left),
    );
    debug!(join = %join.clone().collect().unwrap());
    // // Сортируем по
    // let sort = join.sort_by_exprs(
    //     [
    //         (col(RETENTION_TIME) - lit(key.threshold.retention_time.0)).abs(),
    //         col(MASS_TO_CHARGE),
    //     ],
    //     SortMultipleOptions::new(),
    // );
    // debug!(sort = %sort.clone().collect().unwrap());
    let group = join
        .group_by_stable([col(RETENTION_TIME)])
        .agg([col(SIGNAL)]);
    debug!(group = %group.clone().collect().unwrap());
    let cosine_distance = group.select([
        col(RETENTION_TIME),
        col(SIGNAL)
            .apply(cosine_distance(0), |_, _field| {
                Ok(Field::new(PlSmallStr::EMPTY, DataType::Float64))
            })
            .alias(COSINE_DISTANCE),
    ]);
    debug!(cosine_distance = %cosine_distance.clone().collect().unwrap());
    lazy_frame = lazy_frame.join(
        cosine_distance,
        [col(RETENTION_TIME)],
        [col(RETENTION_TIME)],
        JoinArgs::new(JoinType::Left),
    );
    let exprs = key
        .retention_times
        .iter()
        .map(|&retention_time| col(RETENTION_TIME).eq(retention_time))
        .collect::<Vec<_>>();
    // let exprs = Vec::new();
    // for retention_time in key.retention_times {
    //     //
    // }
    concat_arr(
        key.retention_times
            .iter()
            .map(|&retention_time| col(RETENTION_TIME).eq(retention_time).arg)
            .collect(),
    )?;
    lazy_frame = lazy_frame.select([
        col(RETENTION_TIME),
        col(MASS_SPECTRUM),
        col(META).struct_().with_fields(vec![
            col(COSINE_DISTANCE),
            col(META)
                .struct_()
                .field_by_name(THRESHOLD)
                .and(col(COSINE_DISTANCE).lt(key.threshold.factor.0)),
        ]),
    ]);
    debug!(lazy_frame = %lazy_frame.clone().collect().unwrap());
    Ok(lazy_frame)
}

fn cosine_distance(
    index: usize,
) -> impl Fn(Column) -> PolarsResult<Column> + 'static + Send + Sync {
    move |column| {
        let signal = column.list()?;
        let mut target = signal
            .get_as_series(index)
            .ok_or(polars_err!(oob = index, signal.len()))?
            .f64()?
            .fill_null_with_values(0.0)?
            .into_no_null_iter()
            .collect::<Vec<_>>();
        Ok(signal
            .into_iter()
            .map(|mass_spectrum| {
                let source = mass_spectrum
                    .ok_or(polars_err!(NoData: SIGNAL))?
                    .f64()?
                    .fill_null_with_values(0.0)?
                    .into_no_null_iter()
                    .collect::<Vec<_>>();
                assert_eq!(source.len(), target.len());
                let distance = cosine(&source, &target);
                // Сходство (cos угла) более чем на 99%
                // let threshold = distance < key.threshold.factor.0;
                Ok(Some(distance))
            })
            .collect::<PolarsResult<Float64Chunked>>()?
            .into_column())
    }
}

fn _cosine_distance(column: Column) -> PolarsResult<Column> {
    let signal = column.list()?;
    let mut target = signal
        .get_as_series(0)
        .ok_or(polars_err!(oob = 0, signal.len()))?
        .f64()?
        .fill_null_with_values(f64::EPSILON)?
        .into_no_null_iter()
        .collect::<Vec<_>>();
    // println!("target: {target:?}");
    Ok(signal
        .into_iter()
        .map(|mass_spectrum| {
            let source = mass_spectrum
                .ok_or(polars_err!(NoData: "SIGNAL"))?
                .f64()?
                .fill_null_with_values(f64::EPSILON)?
                .into_no_null_iter()
                .collect::<Vec<_>>();
            let distance = cosine(&source, &target);
            // // Сходство (cos угла) более чем на 75%
            // let threshold = distance < key.threshold.factor.0;
            // if threshold {}
            Ok(Some(distance))
        })
        .collect::<PolarsResult<Float64Chunked>>()?
        .into_column())
}

/// Threshold by cosine distance
fn threshold(key: Key) -> impl Fn(Column) -> PolarsResult<Column> + 'static + Send + Sync {
    const TARGET: &str = formatcp!("{SIGNAL}_right");

    move |column| {
        let r#struct = column.struct_()?;
        let retention_time = r#struct.field_by_name(RETENTION_TIME)?;
        let index = abs(&retention_time.sub(key.threshold.retention_time.0))?
            .arg_min()
            .ok_or(polars_err!(NoData: "RETENTION_TIME"))?;
        let mass_spectrum_series = r#struct.field_by_name(MASS_SPECTRUM)?;
        let mass_spectrum = mass_spectrum_series.list()?;
        let mut target = {
            let series = mass_spectrum
                .get_as_series(index)
                .ok_or(polars_err!(oob = index, mass_spectrum.len()))?;
            let r#struct = series.struct_()?;
            df! {
                MASS_TO_CHARGE => r#struct.field_by_name(MASS_TO_CHARGE)?.round(0, RoundMode::HalfToEven)?.f64()?.clone(),
                SIGNAL => r#struct.field_by_name(SIGNAL)?.f64()?.clone(),
            }?
        };
        Ok(mass_spectrum.into_iter().map(|mass_spectrum| {
            let source = {
                let series = mass_spectrum.ok_or(polars_err!(NoData: "SOURCE"))?;
                let r#struct = series.struct_()?;
                df! {
                    MASS_TO_CHARGE => r#struct.field_by_name(MASS_TO_CHARGE)?.round(0, RoundMode::HalfToEven)?.f64()?.clone(),
                    SIGNAL => r#struct.field_by_name(SIGNAL)?.f64()?.clone(),
                }?
            };
            let join = source.join(
                &target,
                [MASS_TO_CHARGE],
                [MASS_TO_CHARGE],
                JoinArgs::new(JoinType::Full).with_coalesce(JoinCoalesce::CoalesceColumns),
                None,
            )?;
            let a = join[SIGNAL].f64()?.fill_null_with_values(0.0)?.into_no_null_iter().collect::<Vec<_>>();
            let b = join[TARGET]
                .f64()?
                .fill_null_with_values(0.0)?.into_no_null_iter().collect::<Vec<_>>();
            let distance = cosine(&a, &b);
            // Сходство (cos угла) более чем на 75%
            let threshold = distance < key.threshold.factor.0;
            if threshold {
                // target = join.lazy().with_column((col(SIGNAL) + col(TARGET)).median()).collect()?;
            }
            Ok(Some(threshold))
        }).collect::<PolarsResult<BooleanChunked>>()?.into_column())
        // let fields = column.struct_()?.fields_as_series();
        // let tagret = fields[1]
        //     .list()?
        //     .get_as_series(0)
        //     .ok_or(polars_err!(NoData: "TAGRET"))?
        //     .f64()?;
        // Ok(zip(fields[0].list()?, fields[1].list()?).into_iter().map(|(source, tagret)| {
        //     let source = {
        //         let series = source.ok_or(polars_err!(NoData: "SOURCE"))?;
        //         let r#struct = series.struct_()?;
        //         df! {
        //             MASS_TO_CHARGE => r#struct.field_by_name(MASS_TO_CHARGE)?.round(0, RoundMode::HalfToEven)?.f64()?.clone(),
        //             SIGNAL => r#struct.field_by_name(SIGNAL)?.f64()?.clone(),
        //         }?
        //     };
        //     let tagret = {
        //         let series = tagret.ok_or(polars_err!(NoData: "TAGRET"))?;
        //         let r#struct = series.struct_()?;
        //         df! {
        //             MASS_TO_CHARGE => r#struct.field_by_name(MASS_TO_CHARGE)?.round(0, RoundMode::HalfToEven)?.f64()?.clone(),
        //             SIGNAL => r#struct.field_by_name(SIGNAL)?.f64()?.clone(),
        //         }?
        //     };
        //     let join = source.join(
        //         &tagret,
        //         [MASS_TO_CHARGE],
        //         [MASS_TO_CHARGE],
        //         JoinArgs::new(JoinType::Full).with_coalesce(JoinCoalesce::CoalesceColumns),
        //         None,
        //     )?;
        //     let a = join[SIGNAL].f64()?.fill_null_with_values(0.0)?.into_no_null_iter().collect::<Vec<_>>();
        //     let b = join[formatcp!("{SIGNAL}_right")]
        //         .f64()?
        //         .fill_null_with_values(0.0)?.into_no_null_iter().collect::<Vec<_>>();
        //     Ok(Some(cosine(&a, &b)))
        // }).collect::<PolarsResult<Float64Chunked>>()?.into_column())
    }
}

/// Filter and sort threshold
fn filter_and_sort(lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    if key.threshold.filter {
        lazy_frame.filter(col(META).struct_().field_by_name(THRESHOLD))
    } else if key.threshold.sort {
        lazy_frame.sort_by_exprs(
            [col(META).struct_().field_by_name(THRESHOLD)],
            SortMultipleOptions::default()
                .with_maintain_order(true)
                .with_order_reversed(),
        )
    } else {
        lazy_frame
    }
}

#[cfg(test)]
mod test {
    use scirs2::spatial::cosine;

    // 92.0, 93.0, 94.0, 94.0, 95.0, 96.0, 97.0, 98.0, 99.0, 100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0, 111.0, 112.0, 113.0, 114.0, 115.0, 116.0, 117.0, 118.0, 119.0, 120.0, 121.0, 122.0, 123.0, 124.0, 125.0, 126.0, 127.0, 128.0, 129.0, 130.0, 131.0, 132.0, 133.0, 134.0, 135.0, 136.0, 137.0, 138.0, 139.0, 140.0, 141.0, 142.0, 143.0, 144.0, 145.0, 146.0, 147.0, 148.0, 149.0, 150.0, 151.0, 152.0, 153.0, 154.0, 155.0, 156.0, 157.0, 158.0, 159.0, 160.0, 161.0, 162.0, 163.0, 164.0, 165.0, 166.0, 167.0, 168.0, 169.0, 170.0, 171.0, 172.0, 173.0, 174.0, 175.0, 176.0, 177.0, 178.0, 179.0, 180.0, 181.0, 182.0, 183.0, 184.0, 185.0, 186.0, 187.0, 188.0, 189.0, 190.0, 191.0, 192.0, 193.0, 194.0, 195.0, 196.0, 197.0, 198.0, 199.0, 200.0, 201.0, 202.0, 203.0, 204.0, 205.0, 206.0, 207.0, 208.0, 209.0, 210.0, 211.0, 212.0, 213.0, 214.0, 215.0, 216.0, 217.0, 218.0, 219.0, 220.0, 221.0, 222.0, 223.0, 224.0, 225.0, 226.0, 227.0, 228.0, 229.0, 230.0, 231.0, 232.0, 233.0, 234.0, 235.0, 236.0, 237.0, 238.0, 239.0, 240.0, 241.0, 242.0, 243.0, 244.0, 245.0, 246.0, 247.0, 248.0, 249.0, 250.0, 251.0, 252.0, 253.0, 254.0, 255.0, 256.0, 257.0, 258.0, 259.0, 260.0, 261.0, 262.0, 263.0, 264.0, 265.0, 266.0, 267.0, 268.0, 269.0, 270.0, 271.0, 272.0, 273.0, 274.0, 275.0, 276.0, 277.0, 278.0, 279.0, 280.0, 281.0, 282.0, 283.0, 284.0, 285.0, 286.0, 287.0, 288.0, 289.0, 290.0, 291.0, 292.0, 293.0, 294.0, 295.0, 296.0, 297.0, 298.0, 299.0, 300.0, 302.0, 303.0, 304.0, 305.0, 306.0, 307.0, 308.0, 309.0, 310.0, 311.0, 312.0, 313.0, 314.0, 315.0, 317.0, 318.0, 319.0, 320.0, 321.0, 322.0, 323.0, 324.0, 325.0, 326.0, 327.0, 328.0, 330.0, 331.0, 333.0, 334.0, 335.0, 336.0, 337.0, 338.0, 339.0, 340.0, 341.0, 342.0, 346.0, 347.0, 348.0, 349.0, 350.0, 351.0, 352.0, 353.0, 354.0, 355.0, 356.0, 357.0, 366.0, 367.0, 368.0, 369.0, 370.0, 371.0, 373.0, 380.0, 381.0, 382.0, 383.0, 384.0, 385.0, 389.0, 390.0]
    // 92.0, 93.0, 94.0, 95.0, 96.0, 97.0, 98.0, 99.0, 100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0, 111.0, 112.0, 113.0, 114.0, 115.0, 116.0, 117.0, 118.0, 119.0, 120.0, 121.0, 122.0, 123.0, 124.0, 125.0, 126.0, 127.0, 128.0, 129.0, 130.0, 131.0, 132.0, 133.0, 134.0, 135.0, 136.0, 137.0, 138.0, 139.0, 140.0, 141.0, 142.0, 143.0, 144.0, 145.0, 146.0, 147.0, 148.0, 149.0, 150.0, 151.0, 152.0, 153.0, 154.0, 155.0, 156.0, 157.0, 158.0, 159.0, 160.0, 161.0, 162.0, 163.0, 164.0, 165.0, 166.0, 167.0, 168.0, 169.0, 170.0, 171.0, 172.0, 173.0, 174.0, 175.0, 176.0, 177.0, 178.0, 179.0, 180.0, 181.0, 182.0, 183.0, 184.0, 185.0, 186.0, 187.0, 188.0, 189.0, 190.0, 191.0, 192.0, 193.0, 194.0, 195.0, 196.0, 197.0, 198.0, 199.0, 200.0, 201.0, 202.0, 203.0, 204.0, 205.0, 206.0, 207.0, 208.0, 209.0, 210.0, 211.0, 212.0, 213.0, 214.0, 215.0, 216.0, 217.0, 218.0, 219.0, 220.0, 221.0, 222.0, 223.0, 224.0, 225.0, 226.0, 227.0, 228.0, 229.0, 230.0, 231.0, 232.0, 233.0, 234.0, 235.0, 236.0, 237.0, 238.0, 239.0, 240.0, 241.0, 242.0, 243.0, 244.0, 245.0, 246.0, 247.0, 248.0, 249.0, 250.0, 251.0, 252.0, 253.0, 254.0, 255.0, 256.0, 257.0, 258.0, 259.0, 260.0, 261.0, 262.0, 263.0, 264.0, 265.0, 266.0, 267.0, 268.0, 269.0, 270.0, 271.0, 272.0, 273.0, 274.0, 275.0, 276.0, 277.0, 278.0, 279.0, 280.0, 281.0, 282.0, 283.0, 284.0, 285.0, 286.0, 287.0, 288.0, 289.0, 290.0, 291.0, 292.0, 293.0, 294.0, 295.0, 296.0, 297.0, 298.0, 299.0, 300.0, 302.0, 303.0, 304.0, 305.0, 306.0, 307.0, 308.0, 309.0, 310.0, 311.0, 312.0, 313.0, 314.0, 315.0, 317.0, 318.0, 319.0, 320.0, 321.0, 322.0, 323.0, 324.0, 325.0, 326.0, 327.0, 328.0, 330.0, 331.0, 333.0, 334.0, 335.0, 336.0, 337.0, 338.0, 339.0, 340.0, 341.0, 342.0, 346.0, 347.0, 348.0, 349.0, 350.0, 351.0, 352.0, 353.0, 354.0, 355.0, 356.0, 357.0, 366.0, 367.0, 368.0, 369.0, 370.0, 371.0, 373.0, 380.0, 381.0, 382.0, 383.0, 384.0, 385.0, 389.0, 390.0]

    const A: [f64; 316] = [
        0.0, 246.0, 190.0, 315.0, 319.0, 557.0, 246.0, 391.0, 0.0, 362.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 219.0, 305.0, 0.0, 253.0, 0.0, 0.0, 0.0, 566.0, 0.0, 415.0, 0.0, 340.0, 0.0, 173.0,
        173.0, 215.0, 237.0, 174.0, 256.0, 158.0, 0.0, 0.0, 0.0, 0.0, 0.0, 337.0, 0.0, 0.0, 197.0,
        197.0, 153.0, 224.0, 0.0, 0.0, 0.0, 0.0, 0.0, 154.0, 164.0, 0.0, 0.0, 0.0, 176.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 217.0, 224.0, 0.0, 165.0, 0.0, 168.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 302.0, 0.0, 291.0, 161.0, 611.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 381.0, 0.0, 0.0, 224.0, 0.0, 1354.0, 287.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        197.0, 0.0, 0.0, 0.0, 0.0, 155.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 455.0, 0.0,
        662.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 213.0, 169.0, 0.0, 0.0, 0.0, 1496.0, 318.0,
        223.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 168.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 183.0, 186.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 205.0,
        158.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 221.0, 0.0, 0.0, 0.0, 0.0, 168.0, 264.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 340.0, 0.0, 160.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 3218.0, 997.0, 535.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        0.0,
        // 0.0,
    ];

    const B: [f64; 316] = [
        0.0, 434.0, 0.0, 1109.0, 661.0, 13129.0, 2397.0, 5092.0, 258.0, 7434.0, 294.0, 192.0, 0.0,
        0.0, 0.0, 418.0, 343.0, 1333.0, 1296.0, 6524.0, 1567.0, 2884.0, 216.0, 1566.0, 26957.0,
        8484.0, 750.0, 185.0, 244.0, 520.0, 0.0, 1366.0, 784.0, 3595.0, 3611.0, 1920.0, 408.0,
        22295.0, 4642.0, 407.0, 0.0, 285.0, 0.0, 252.0, 156.0, 1214.0, 401.0, 2335.0, 1956.0,
        249.0, 0.0, 5137.0, 458.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 523.0, 679.0, 722.0, 150.0,
        0.0, 1864.0, 174.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 346.0, 181.0, 0.0, 0.0, 4996.0,
        394.0, 0.0, 0.0, 0.0, 0.0, 204.0, 0.0, 158.0, 0.0, 0.0, 0.0, 0.0, 0.0, 18607.0, 1631.0,
        241.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 244.0, 189.0, 9980.0, 1120.0, 3278.0, 305.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1593.0, 240.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 922.0, 271.0, 200.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 209.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ];

    #[test]
    fn test() {
        let d = cosine(&A, &B);
        println!("d: {d}");
    }
}
