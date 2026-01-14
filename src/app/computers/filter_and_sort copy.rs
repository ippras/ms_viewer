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
use std::ops::Sub;
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
    let join = cross_join
        .join(
            explode,
            [col(RETENTION_TIME), col(MASS_TO_CHARGE)],
            [col(RETENTION_TIME), col(MASS_TO_CHARGE)],
            JoinArgs::new(JoinType::Left),
        )
        .sort_by_exprs(
            [
                (col(RETENTION_TIME) - lit(key.threshold.retention_time.0 * MINUTES))
                    .abs()
                    .arg_sort(false, false),
                col(MASS_TO_CHARGE),
            ],
            SortMultipleOptions::new(),
        );
    println!("E5: {}", join.clone().collect().unwrap());
    let group = join
        .group_by_stable([col(RETENTION_TIME)])
        .agg([col(SIGNAL)]);
    println!("E6: {}", group.clone().collect().unwrap());
    let cosine_distance = group.with_column(
        col(SIGNAL)
            .apply(cosine_distance(key), |_, _field| {
                Ok(Field::new(PlSmallStr::EMPTY, DataType::Boolean))
            })
            .alias("CosineDistance"),
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
            .and(col("CosineDistance")),
    ])]);
    println!("E8: {}", lazy_frame.clone().collect().unwrap());
    Ok(lazy_frame)
}

// fn compute_cd(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
// }

fn mass_to_charge(expr: Expr) -> Expr {
    expr.struct_()
        .field_by_name(MASS_TO_CHARGE)
        .round(0, RoundMode::HalfToEven)
}

fn indexed_element(index: Option<u64>) -> Expr {
    match index {
        Some(index) => element().get(index),
        None => element(),
    }
}

// fn cosine_distance(a: Expr, b: Expr) -> Expr {
//     lit(1) - (a.clone() * b.clone()).sum() / (a.pow(2).sum().sqrt() * b.pow(2).sum().sqrt())
// }
fn cosine_distance(key: Key) -> impl Fn(Column) -> PolarsResult<Column> + 'static + Send + Sync {
    move |column| {
        let signal = column.list()?;
        let mut target = signal
            .get_as_series(0)
            .ok_or(polars_err!(oob = 0, signal.len()))?
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
                // Сходство (cos угла) более чем на 75%
                let threshold = distance < key.threshold.factor.0;
                if threshold {}
                Ok(Some(threshold))
            })
            .collect::<PolarsResult<BooleanChunked>>()?
            .into_column())
    }
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
