use crate::{
    app::panes::settings::{Settings, Sort, TimeUnits},
    r#const::*,
    utils::hash::HashedDataFrame,
};
use egui::util::cache::{ComputerMut, FrameCache};
use polars::{frame::DataFrame, prelude::*};
use polars_ext::column;
use std::hash::{Hash, Hasher};
use tracing::{error, trace, warn};
use uom::si::{
    f64::Time,
    time::{millisecond, minute, second},
};

const MINUTES: f64 = 60_000.0;

/// Table computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Table computer
#[derive(Default)]
pub(crate) struct Computer;

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key<'_>) -> Value {
        error!(?key.frame.data_frame);
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        // Filter nulls
        if key.filter_null {
            lazy_frame = lazy_frame.drop_nulls(Some(cols([MASS_TO_CHARGE, SIGNAL])));
            // lazy_frame = lazy_frame.filter(col(MASS_SPECTRUM).list().len().neq(lit(0)));
        }
        // Normalize signal
        if key.normalize_signal {
            lazy_frame = lazy_frame.with_column(col(SIGNAL).cast(DataType::Float64) / max(SIGNAL));
        }
        // Sort
        lazy_frame = match key.sort {
            Sort::RetentionTime => retention_time(lazy_frame, key),
            Sort::MassToCharge => mass_to_charge(lazy_frame, key),
        };
        println!("lazy_frame gg: {}", lazy_frame.clone().collect().unwrap());
        let options = RollingOptionsFixedWindow {
            window_size: 3,
            min_periods: 3,
            weights: None,
            center: true,
            fn_params: None,
        };
        lazy_frame = lazy_frame
            .with_columns([col(RETENTION_TIME).alias("x"), col("Signal.Sum").alias("y")])
            .with_columns([
                (col("x") * col("y"))
                    .rolling_mean(options.clone())
                    .alias("xy_mean"),
                col("x").rolling_mean(options.clone()).alias("x_mean"),
                col("y").rolling_mean(options.clone()).alias("y_mean"),
                col("x").rolling_std(options.clone()).alias("x_std"),
                col("y").rolling_std(options.clone()).alias("y_std"),
            ])
            .with_column((col("xy_mean") - col("x_mean") * col("y_mean")).alias("cov_xy"))
            .with_column((col("cov_xy") / (col("x_std") * col("y_std"))).alias("Correlation"))
            .with_column((col("Correlation") * (col("y_std") / col("x_std"))).alias("Slope"))
            .with_column((col("y_mean") - col("Slope") * col("x_mean")).alias("Intercept"))
            .sort([RETENTION_TIME], Default::default());
        println!("lazy_frame gg1: {}", lazy_frame.clone().collect().unwrap());
        let data_frame = lazy_frame.collect().unwrap();
        trace!(?data_frame);
        HashedDataFrame::new(data_frame).unwrap()
    }
}

/// Table key
#[derive(Clone, Copy, Hash, Debug)]
pub struct Key<'a> {
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) explode: bool,
    pub(crate) filter_null: bool,
    pub(crate) normalize_signal: bool,
    pub(crate) sort: Sort,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &Settings) -> Self {
        Self {
            frame,
            explode: settings.explode,
            filter_null: settings.filter_null,
            normalize_signal: settings.signal.normalize,
            sort: settings.sort,
        }
    }
}

/// Table value
type Value = HashedDataFrame;

fn mass_to_charge(mut lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    trace!(lazy_data_frame =? lazy_frame.clone().collect());
    lazy_frame = lazy_frame
        .sort([RETENTION_TIME], Default::default())
        .group_by([col(MASS_TO_CHARGE).round(2, RoundMode::HalfToEven)])
        .agg([as_struct(vec![col(RETENTION_TIME), col(SIGNAL)]).alias("ExtractedIonChromatogram")]);
    if !key.explode {
        lazy_frame = lazy_frame.with_columns([
            col("ExtractedIonChromatogram")
                .list()
                .len()
                .name()
                .suffix(".Count"),
            col("ExtractedIonChromatogram")
                .list()
                .eval(element().struct_().field_by_name(RETENTION_TIME))
                .list()
                .min()
                .alias("RetentionTime.Min"),
            col("ExtractedIonChromatogram")
                .list()
                .eval(element().struct_().field_by_name(RETENTION_TIME))
                .list()
                .max()
                .alias("RetentionTime.Max"),
            col("ExtractedIonChromatogram")
                .list()
                .eval(element().struct_().field_by_name(SIGNAL))
                .list()
                .min()
                .alias("Signal.Min"),
            col("ExtractedIonChromatogram")
                .list()
                .eval(element().struct_().field_by_name(SIGNAL))
                .list()
                .max()
                .alias("Signal.Max"),
            col("ExtractedIonChromatogram")
                .list()
                .eval(element().struct_().field_by_name(SIGNAL))
                .list()
                .sum()
                .alias("Signal.Sum"),
        ]);
    }
    lazy_frame = lazy_frame.sort([MASS_TO_CHARGE], Default::default());
    lazy_frame
}

fn retention_time(mut lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    lazy_frame = lazy_frame
        .sort([MASS_TO_CHARGE], Default::default())
        .group_by([col(RETENTION_TIME)])
        .agg([as_struct(vec![col(MASS_TO_CHARGE), col(SIGNAL)]).alias(MASS_SPECTRUM)]);
    if !key.explode {
        lazy_frame = lazy_frame
            .with_columns([
                col(RETENTION_TIME)
                    .cast(DataType::Duration(TimeUnit::Milliseconds))
                    .to_physical()
                    / lit(MINUTES),
                col(MASS_SPECTRUM).list().len().name().suffix(".Count"),
                col(MASS_SPECTRUM)
                    .list()
                    .eval(element().struct_().field_by_name(MASS_TO_CHARGE))
                    .list()
                    .min()
                    .alias("MassToCharge.Min"),
                col(MASS_SPECTRUM)
                    .list()
                    .eval(element().struct_().field_by_name(MASS_TO_CHARGE))
                    .list()
                    .max()
                    .alias("MassToCharge.Max"),
                col(MASS_SPECTRUM)
                    .list()
                    .eval(element().struct_().field_by_name(SIGNAL))
                    .list()
                    .min()
                    .alias("Signal.Min"),
                col(MASS_SPECTRUM)
                    .list()
                    .eval(element().struct_().field_by_name(SIGNAL))
                    .list()
                    .max()
                    .alias("Signal.Max"),
                col(MASS_SPECTRUM)
                    .list()
                    .eval(element().struct_().field_by_name(SIGNAL))
                    .list()
                    .sum()
                    .alias("Signal.Sum"),
            ])
            .sort([RETENTION_TIME], Default::default())
            .with_column(col("Signal.Sum").peak_max().alias("PeakMax"))
            .filter(col("PeakMax"));
    }
    lazy_frame = lazy_frame.sort([RETENTION_TIME], Default::default());
    lazy_frame
}

// df.with_columns([
//     (pl.col("x") * pl.col("y")).rolling_mean(window_size).alias("xy_mean"),
//     pl.col("x").rolling_mean(window_size).alias("x_mean"),
//     pl.col("y").rolling_mean(window_size).alias("y_mean"),
//     pl.col("x").rolling_std(window_size).alias("x_std"),
//     pl.col("y").rolling_std(window_size).alias("y_std"),
// ]).with_columns(
//     (pl.col("xy_mean") - pl.col("x_mean") * pl.col("y_mean")).alias("cov_xy")
// ).with_columns(
//     (pl.col("cov_xy") / (pl.col("x_std") * pl.col("y_std"))).alias("correlation")
// ).with_columns(
//     (pl.col("correlation") * (pl.col("y_std") / pl.col("x_std"))).alias("slope"),
// ).with_columns(
//     (pl.col("y_mean") - pl.col("slope") * pl.col("x_mean")).alias("intercept"),
// )

// pub fn retention_time(units: TimeUnits) -> impl Fn(&Series) -> PolarsResult<Series> {
//     move |series| {
//         Ok(series
//             .cast(&DataType::Float64)?
//             .f64()?
//             .iter()
//             .map(|value| {
//                 let time = Time::new::<millisecond>(value?);
//                 Some(match units {
//                     TimeUnits::Millisecond => time.get::<millisecond>(),
//                     TimeUnits::Second => time.get::<second>(),
//                     TimeUnits::Minute => time.get::<minute>(),
//                 })
//             })
//             .collect::<Float64Chunked>()
//             .into_series())
//     }
// }

// fn retention_time(retention_time: RetentionTime) -> Expr {
//     // element().struct_().field_by_name(SIGNAL)
// }

// fn signal() -> Expr {
//     element().struct_().field_by_name(SIGNAL)
// }
