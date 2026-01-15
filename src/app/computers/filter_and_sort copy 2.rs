use crate::{
    app::{
        computers::MINUTES,
        states::pane::settings::{Settings, Threshold},
    },
    r#const::*,
    utils::hash::HashedDataFrame,
};
use const_format::formatcp;
use egui::util::cache::{ComputerMut, FrameCache};
use polars::prelude::*;
use scirs2::spatial::cosine;
use std::{f64, ops::Sub};
use tracing::{instrument, trace};

/// Peak computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Peak computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    #[instrument(skip(self), err)]
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
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &Settings) -> Self {
        Self {
            frame,
            threshold: settings.threshold,
        }
    }
}

/// Peak value
type Value = HashedDataFrame;

/// Format
fn compute(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    // let index = (col(RETENTION_TIME) - lit(key.threshold.retention_time.0 * MINUTES))
    //     .abs()
    //     .arg_min();
    // let index = (col(RETENTION_TIME) - lit(key.threshold.retention_time.0 * MINUTES))
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
    println!("E0: {}", lazy_frame.clone().collect().unwrap());
    let retention_time = lazy_frame.clone().select([col(RETENTION_TIME)]);
    println!("E1: {}", retention_time.clone().collect().unwrap());
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
        .with_column(col(MASS_TO_CHARGE).round(0, RoundMode::HalfToEven));
    println!("E2: {}", explode.clone().collect().unwrap());
    let mass_to_charge = explode.clone().select([col(MASS_TO_CHARGE).unique()]);
    println!("E3: {}", mass_to_charge.clone().collect().unwrap());
    // Получаем все возможные пары (RETENTION_TIME, MASS_TO_CHARGE)
    let cross_join = retention_time.cross_join(mass_to_charge, None);
    println!("E4: {}", cross_join.clone().collect().unwrap());
    // Сопоставляем с SIGNAL
    let join = cross_join.join(
        explode,
        [col(RETENTION_TIME), col(MASS_TO_CHARGE)],
        [col(RETENTION_TIME), col(MASS_TO_CHARGE)],
        JoinArgs::new(JoinType::Semi),
    );
    println!("E5.1: {}", join.clone().collect().unwrap());
    let sort = join
        .with_column(
            (col(RETENTION_TIME) - lit(key.threshold.retention_time.0 * MINUTES))
                .abs()
                .alias("Abs"),
        )
        .sort_by_exprs(
            [col("Abs"), col(MASS_TO_CHARGE)],
            SortMultipleOptions::new(),
        );
    // (col(RETENTION_TIME) - lit(key.threshold.retention_time.0 * MINUTES))
    //     .abs()
    //     .arg_sort(false, false),
    println!("E5.2: {}", sort.clone().collect().unwrap());
    let group = sort
        .group_by_stable([col(RETENTION_TIME)])
        .agg([col(SIGNAL)]);
    println!("E6: {}", group.clone().collect().unwrap());
    let cosine_distance = group.with_column(
        col(SIGNAL)
            .apply(cosine_distance(0), |_, _field| {
                Ok(Field::new(PlSmallStr::EMPTY, DataType::Float64))
            })
            .alias(COSINE_DISTANCE),
    );
    println!("E7: {}", cosine_distance.clone().collect().unwrap());
    lazy_frame = lazy_frame.join(
        cosine_distance,
        [col(RETENTION_TIME)],
        [col(RETENTION_TIME)],
        JoinArgs::new(JoinType::Left),
    );
    lazy_frame = lazy_frame.with_columns([col(META).struct_().with_fields(vec![
        col(META)
            .struct_()
            .field_by_name(THRESHOLD)
            .and(col(COSINE_DISTANCE).lt(key.threshold.factor.0)),
    ])]);
    println!("E8: {}", lazy_frame.clone().collect().unwrap());
    Ok(lazy_frame)
}

// fn cosine_distance(a: Expr, b: Expr) -> Expr { lit(1) - (a.clone() *
//     b.clone()).sum() / (a.pow(2).sum().sqrt() * b.pow(2).sum().sqrt()) }

// 374465/60000=6.2410833333333333333

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
                    .ok_or(polars_err!(NoData: "SIGNAL"))?
                    .f64()?
                    .fill_null_with_values(0.0)?
                    .into_no_null_iter()
                    .collect::<Vec<_>>();
                let distance = cosine(&source, &target);
                // // Сходство (cos угла) более чем на 75%
                // let threshold = distance < key.threshold.factor.0;
                // if threshold {}
                // if distance.is_nan() {
                //     println!("source: {source:?}");
                //     println!("target: {target:?}");
                // }
                Ok(Some(distance))
            })
            .collect::<PolarsResult<Float64Chunked>>()?
            .into_column())
    }
}

// target: [0.0, 0.0, 0.0, 0.0, 0.0, 553.0, 0.0, 313.0, 0.0, 230.0, 0.0, 0.0,
// 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 247.0, 0.0, 151.0, 0.0, 0.0, 2517.0,
// 246.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 173.0, 190.0, 0.0, 0.0, 2071.0,
// 235.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 285.0, 0.0, 0.0, 0.0, 0.0, 0.0,
// 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
// 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 171.0, 0.0, 0.0,
// 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 551.0, 0.0, 0.0, 0.0,
// 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 157.0, 0.0, 0.0, 0.0, 0.0,
// 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
// 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 200.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
// 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 450.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
// 1125.0, 166.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
// 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
// 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 156.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
// 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
// 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
// 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
// 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
// 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
// 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
// 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
// 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
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
        let index = abs(&retention_time.sub(key.threshold.retention_time.0 * MINUTES))?
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
