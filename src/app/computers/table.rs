use crate::{
    app::states::settings::{Settings, Sort, TimeUnits},
    r#const::*,
    utils::hash::HashedDataFrame,
};
use const_format::formatcp;
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
        // Compute
        lazy_frame = compute(lazy_frame, key);
        println!("lazy_frame gg: {}", lazy_frame.clone().collect().unwrap());
        lazy_frame = rolling(lazy_frame, key);
        println!("lazy_frame gg1: {}", lazy_frame.clone().collect().unwrap());
        lazy_frame = filter(lazy_frame, key);
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
    pub(crate) min_periods: usize,
    pub(crate) normalize_signal: bool,
    pub(crate) peak_max: [bool; 2],
    pub(crate) peak_min: [bool; 2],
    pub(crate) sort: Sort,
    pub(crate) window_size: usize,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &Settings) -> Self {
        Self {
            frame,
            explode: settings.explode,
            filter_null: settings.filter_null,
            min_periods: settings.min_periods,
            normalize_signal: settings.signal.normalize,
            peak_max: settings.peak_max,
            peak_min: settings.peak_min,
            sort: settings.sort,
            window_size: settings.window_size,
        }
    }
}

/// Table value
type Value = HashedDataFrame;

fn compute(lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    match key.sort {
        Sort::RetentionTime => retention_time(lazy_frame, key),
        Sort::MassToCharge => mass_to_charge(lazy_frame, key),
    }
}

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
        .agg([as_struct(vec![
            col(MASS_TO_CHARGE),
            col(SIGNAL).cast(DataType::Float64),
        ])
        .alias(MASS_SPECTRUM)]);
    if !key.explode {
        lazy_frame = lazy_frame.with_columns([
            col(RETENTION_TIME)
                .cast(DataType::Duration(TimeUnit::Milliseconds))
                .to_physical()
                / lit(MINUTES),
            col(MASS_SPECTRUM)
                .list()
                .len()
                .name()
                .prefix("_")
                .name()
                .suffix(formatcp!(".{COUNT}")),
            col(MASS_SPECTRUM)
                .list()
                .eval(element().struct_().field_by_name(MASS_TO_CHARGE))
                .list()
                .min()
                .alias(formatcp!("_{MASS_TO_CHARGE}.{MIN}")),
            col(MASS_SPECTRUM)
                .list()
                .eval(element().struct_().field_by_name(MASS_TO_CHARGE))
                .list()
                .max()
                .alias(formatcp!("_{MASS_TO_CHARGE}.{MAX}")),
            col(MASS_SPECTRUM)
                .list()
                .eval(element().struct_().field_by_name(SIGNAL))
                .list()
                .min()
                .alias(formatcp!("_{SIGNAL}.{MIN}")),
            col(MASS_SPECTRUM)
                .list()
                .eval(element().struct_().field_by_name(SIGNAL))
                .list()
                .max()
                .alias(formatcp!("_{SIGNAL}.{MAX}")),
            col(MASS_SPECTRUM)
                .list()
                .eval(element().struct_().field_by_name(SIGNAL))
                .list()
                .sum()
                .alias(formatcp!("_{SIGNAL}.{SUM}")),
        ]);
    }
    lazy_frame.sort([RETENTION_TIME], Default::default())
}

fn rolling(lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    let options = RollingOptionsFixedWindow {
        window_size: key.window_size,
        min_periods: key.min_periods,
        center: true,
        ..Default::default()
    };
    let x = col(RETENTION_TIME);
    let y = col(formatcp!("_{SIGNAL}.{SUM}"));
    lazy_frame
        .with_columns([
            (x.clone() * y.clone())
                .rolling_mean(options.clone())
                .alias(formatcp!("_xy.{ROLLING}.{MEAN}")),
            x.clone()
                .rolling_mean(options.clone())
                .alias(formatcp!("_x.{ROLLING}.{MEAN}")),
            y.clone()
                .rolling_mean(options.clone())
                .alias(formatcp!("_y.{ROLLING}.{MEAN}")),
            x.rolling_std(options.clone())
                .alias(formatcp!("_x.{ROLLING}.{STANDARD_DEVIATION}")),
            y.rolling_std(options.clone())
                .alias(formatcp!("_y.{ROLLING}.{STANDARD_DEVIATION}")),
        ])
        .with_column(
            (col(formatcp!("_xy.{ROLLING}.{MEAN}"))
                - col(formatcp!("_x.{ROLLING}.{MEAN}")) * col(formatcp!("_y.{ROLLING}.{MEAN}")))
            .alias("_xy.cov"),
        )
        .with_column(
            (col("_xy.cov")
                / (col(formatcp!("_x.{ROLLING}.{STANDARD_DEVIATION}"))
                    * col(formatcp!("_y.{ROLLING}.{STANDARD_DEVIATION}"))))
            .alias("Correlation"),
        )
        .with_column(
            (col("Correlation")
                * (col(formatcp!("_y.{ROLLING}.{STANDARD_DEVIATION}"))
                    / col(formatcp!("_x.{ROLLING}.{STANDARD_DEVIATION}"))))
            .alias("Slope"),
        )
        .with_column(
            (col(formatcp!("_y.{ROLLING}.{MEAN}"))
                - col("Slope") * col(formatcp!("_x.{ROLLING}.{MEAN}")))
            .alias("Intercept"),
        )
        .sort([RETENTION_TIME], Default::default())
}

fn filter(lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    let expr = col(formatcp!("_{SIGNAL}.{SUM}"));
    lazy_frame.with_column(
        match (key.peak_min[0], key.peak_max[0]) {
            (false, true) => expr.clone().peak_max().and(expr.clone().gt(expr.median())),
            (true, false) => expr.peak_min(),
            (true, true) => expr.clone().peak_max().or(expr.peak_min()),
            (false, false) => lit(true),
        }
        .alias("_Filter"),
    )
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
