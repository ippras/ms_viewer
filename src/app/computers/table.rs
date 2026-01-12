use crate::{
    app::states::pane::settings::{
        Rolling, Settings, Sort, Threshold,
        mass_spectrum::{MassSpectrum, Sort as MassSpectrumSort},
    },
    r#const::*,
    utils::hash::HashedDataFrame,
};
use const_format::formatcp;
use egui::util::cache::{ComputerMut, FrameCache};
use polars::prelude::*;
use polars_ext::expr::ExprExt;
use scirs2::spatial::cosine;
use std::{f64::EPSILON, iter::zip};
use tracing::{error, trace};
// use uom::si::{
//     f64::Time,
//     time::{millisecond, minute, second},
// };

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
        // Cast
        lazy_frame = lazy_frame.with_columns([
            col(SIGNAL).cast(DataType::Float64),
            col(MASS_TO_CHARGE).cast(DataType::Float64),
        ]);
        // let mut signal = col(SIGNAL).cast(DataType::Float64);
        // if key.normalize_signal {
        //     signal = signal / max(SIGNAL)
        // }
        // lazy_frame = lazy_frame.with_column(signal.precision(key.precision, key.significant));
        // // Mass to charge
        // lazy_frame =
        //     lazy_frame.with_column(col(MASS_TO_CHARGE).precision(key.precision, key.significant));

        // Compute
        println!("lazy_frame T0: {}", lazy_frame.clone().collect().unwrap());
        lazy_frame = compute(lazy_frame, key);
        println!("lazy_frame T1: {}", lazy_frame.clone().collect().unwrap());
        lazy_frame = meta(lazy_frame, key).unwrap();
        println!("lazy_frame T2: {}", lazy_frame.clone().collect().unwrap());
        lazy_frame = rolling(lazy_frame, key);
        println!("lazy_frame T3: {}", lazy_frame.clone().collect().unwrap());
        lazy_frame = threshold(lazy_frame, key);
        // println!("lazy_frame T4: {}", lazy_frame.clone().collect().unwrap());
        lazy_frame = filter_and_sort(lazy_frame, key);
        // println!("lazy_frame T5: {}", lazy_frame.clone().collect().unwrap());
        // Format
        lazy_frame = format(lazy_frame, key);
        // lazy_frame = lazy_frame.with_column(col(MASS_SPECTRUM).list().eval(
        //     element().struct_().with_fields(vec![
        //         element()
        //             .struct_()
        //             .field_by_name(MASS_TO_CHARGE)
        //             .precision(key.precision, key.significant),
        //         element()
        //             .struct_()
        //             .field_by_name(SIGNAL)
        //             .precision(key.precision, key.significant),
        //     ]),
        // ));
        // println!("lazy_frame T6: {}", lazy_frame.clone().collect().unwrap());
        let data_frame = lazy_frame.collect().unwrap();
        trace!(?data_frame);
        HashedDataFrame::new(data_frame).unwrap()
    }
}

/// Table key
#[derive(Clone, Copy, Hash, Debug)]
pub struct Key<'a> {
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) percent: bool,
    pub(crate) precision: usize,
    pub(crate) significant: bool,
    pub(crate) explode: bool,
    pub(crate) filter_null: bool,
    pub(crate) normalize_signal: bool,
    pub(crate) sort: Sort,
    pub(crate) rolling: Rolling,
    pub(crate) mass_spectrum: MassSpectrum,
    pub(crate) threshold: Threshold,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &Settings) -> Self {
        Self {
            frame,
            percent: settings.percent,
            precision: settings.precision,
            significant: settings.significant,
            explode: settings.explode,
            filter_null: settings.filter_null,
            normalize_signal: settings.signal.normalize,
            sort: settings.sort,
            mass_spectrum: settings.mass_spectrum,
            rolling: settings.rolling,
            threshold: settings.threshold,
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
        .agg([as_struct(vec![col(RETENTION_TIME), col(SIGNAL)]).alias(EIC)]);
    if !key.explode {
        lazy_frame = lazy_frame.with_columns([
            col(EIC).list().len().name().suffix(".Count"),
            col(EIC)
                .list()
                .eval(element().struct_().field_by_name(RETENTION_TIME))
                .list()
                .min()
                .alias(formatcp!("{RETENTION_TIME}.{MIN}")),
            col(EIC)
                .list()
                .eval(element().struct_().field_by_name(RETENTION_TIME))
                .list()
                .max()
                .alias(formatcp!("{RETENTION_TIME}.{MAX}")),
            col(EIC)
                .list()
                .eval(element().struct_().field_by_name(SIGNAL))
                .list()
                .min()
                .alias(formatcp!("{SIGNAL}.{MIN}")),
            col(EIC)
                .list()
                .eval(element().struct_().field_by_name(SIGNAL))
                .list()
                .max()
                .alias(formatcp!("{SIGNAL}.{MAX}")),
            col(EIC)
                .list()
                .eval(element().struct_().field_by_name(SIGNAL))
                .list()
                .sum()
                .alias(formatcp!("{SIGNAL}.{SUM}")),
        ]);
    }
    lazy_frame = lazy_frame.sort([MASS_TO_CHARGE], Default::default());
    lazy_frame
}

fn retention_time(mut lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    // Normalize signal
    if key.normalize_signal {
        lazy_frame = lazy_frame.with_column(col(SIGNAL) / max(SIGNAL).over([RETENTION_TIME]));
    }
    // Sort
    match key.mass_spectrum.sort {
        MassSpectrumSort::MassToCharge => {
            lazy_frame = lazy_frame.sort(
                [RETENTION_TIME, MASS_TO_CHARGE],
                SortMultipleOptions::default(),
            );
        }
        MassSpectrumSort::Signal => {
            lazy_frame = lazy_frame.sort(
                [RETENTION_TIME, SIGNAL, MASS_TO_CHARGE],
                SortMultipleOptions::default().with_order_descending_multi([false, true, false]),
            );
        }
    }
    // Group
    lazy_frame
        .group_by_stable([col(RETENTION_TIME)])
        .agg([as_struct(vec![col(MASS_TO_CHARGE), col(SIGNAL)]).alias(MASS_SPECTRUM)])
}

fn meta(lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    // Поиск Base Peak m/z
    let base_peak = element()
        .filter((lit(1) - element().struct_().field_by_name(SIGNAL)).lt(EPSILON))
        .first();
    // Поиск кандидата на Молекулярный ион (самая большая масса с сигналом > 1%
    // от базы) В статьях сказано, что у полиенов M+ очень мал, поэтому порог
    // низкий (1 из 100)
    let molecular_peak = element().get(
        element()
            .struct_()
            .field_by_name(MASS_TO_CHARGE)
            .filter(element().struct_().field_by_name(SIGNAL).gt(0.01))
            .arg_max(),
    );
    let ion55 = ion(55);
    let ion67 = ion(67);
    let ion74 = ion(74);
    let ion79 = ion(79);
    let ion81 = ion(81);
    let ion87 = ion(87);
    let ion91 = ion(91);
    let ion108 = ion(108);
    let ion150 = ion(150);
    let base = base_peak.clone().struct_().field_by_name(SIGNAL);
    // 1. Проверка на Насыщенные (Saturated). Критерий: Базовый пик 74, есть 87
    let is_saturated = (base.clone() - ion74.clone())
        .lt(0.5)
        .or(ion74.clone().gt(0.8))
        .and(ion87.clone().gt(0.1));
    // 2. Проверка на Моноеновые (Monoenoic). Критерий: Базовый пик 55, пик 74 есть, но не доминирует.
    let is_monoenoic = (base.clone() - ion55.clone()).lt(0.5);
    // Дополнительная проверка: потеря метанола [M-32] (здесь упрощенно без расчета M)
    // 3. Проверка на Диеновые (Dienoic). Критерий: Базовый пик 67 или 81
    let is_dienoic = (base.clone() - ion67.clone())
        .lt(0.5)
        .or((base.clone() - ion81.clone()).lt(0.5));
    // 4. Проверка на Полиеновые (Polyenoic). Критерий: Базовый пик 79, наличие 91 (тропилий), маленький 74
    let is_polyenoic = (base.clone() - ion79.clone())
        .lt(0.5)
        .or((base.clone() - ion81.clone()).lt(0.5));
    let is_tropylium = ion91.clone().gt(0.2);
    // Проверка Omega-маркеров
    let is_omega_3 = ion108.clone().gt(0.1);
    let is_omega_6 = ion150.clone().gt(0.1);
    Ok(lazy_frame.with_column(
        col(MASS_SPECTRUM)
            .list()
            .agg(as_struct(vec![
                // Statistics
                element().len().alias(formatcp!("{MASS_SPECTRUM}.{LEN}")),
                element()
                    .struct_()
                    .field_by_name(MASS_TO_CHARGE)
                    .min()
                    .alias(formatcp!("{MASS_TO_CHARGE}.{MIN}")),
                element()
                    .struct_()
                    .field_by_name(MASS_TO_CHARGE)
                    .max()
                    .alias(formatcp!("{MASS_TO_CHARGE}.{MAX}")),
                element()
                    .struct_()
                    .field_by_name(SIGNAL)
                    .min()
                    .alias(formatcp!("{SIGNAL}.{MIN}")),
                element()
                    .struct_()
                    .field_by_name(SIGNAL)
                    .max()
                    .alias(formatcp!("{SIGNAL}.{MAX}")),
                element()
                    .struct_()
                    .field_by_name(SIGNAL)
                    .sum()
                    .alias(formatcp!("{SIGNAL}.{SUM}")),
                (element().struct_().field_by_name(SIGNAL)
                    / element().struct_().field_by_name(SIGNAL).sum())
                .min()
                .alias(formatcp!("{SIGNAL}.{MIN}.{NORMALIZED}")),
                (element().struct_().field_by_name(SIGNAL)
                    / element().struct_().field_by_name(SIGNAL).sum())
                .max()
                .alias(formatcp!("{SIGNAL}.{MAX}.{NORMALIZED}")),
                // Find
                base_peak.alias(formatcp!("{MASS_SPECTRUM}.BasePeak")),
                molecular_peak.alias(formatcp!("{MASS_SPECTRUM}.MolecularPeak")),
                ion55.alias(formatcp!("{SIGNAL}.Ion55")),
                ion67.alias(formatcp!("{SIGNAL}.Ion67")),
                ion74.alias(formatcp!("{SIGNAL}.Ion74")),
                ion79.alias(formatcp!("{SIGNAL}.Ion79")),
                ion81.alias(formatcp!("{SIGNAL}.Ion81")),
                ion87.alias(formatcp!("{SIGNAL}.Ion87")),
                ion91.alias(formatcp!("{SIGNAL}.Ion91")),
                ion108.alias(formatcp!("{SIGNAL}.Ion108")),
                ion150.alias(formatcp!("{SIGNAL}.Ion150")),
                // Check
                is_saturated.alias(formatcp!("{MASS_SPECTRUM}.IsSaturated")),
                is_monoenoic.alias(formatcp!("{MASS_SPECTRUM}.IsMonoenoic")),
                is_dienoic.alias(formatcp!("{MASS_SPECTRUM}.IsDienoic")),
                is_polyenoic.alias(formatcp!("{MASS_SPECTRUM}.IsPolyenoic")),
                is_tropylium.alias(formatcp!("{MASS_SPECTRUM}.IsTropylium")),
                is_omega_3.alias(formatcp!("{MASS_SPECTRUM}.IsOmega-3")),
                is_omega_6.alias(formatcp!("{MASS_SPECTRUM}.IsOmega-6")),
            ]))
            .alias(META),
    ))
    // Ok(lazy_frame.with_columns([
    //     col(MASS_SPECTRUM)
    //         .list()
    //         .len()
    //         .alias(formatcp!("_{MASS_SPECTRUM}.{COUNT}")),
    //     // col(MASS_SPECTRUM)
    //     //     .list()
    //     //     .agg(element().struct_().field_by_name(MASS_TO_CHARGE).min())
    //     //     .alias(formatcp!("_{MASS_TO_CHARGE}.{MIN}")),
    //     // col(MASS_SPECTRUM)
    //     //     .list()
    //     //     .agg(element().struct_().field_by_name(MASS_TO_CHARGE).max())
    //     //     .alias(formatcp!("_{MASS_TO_CHARGE}.{MAX}")),
    //     // col(MASS_SPECTRUM)
    //     //     .list()
    //     //     .agg(element().struct_().field_by_name(SIGNAL).min())
    //     //     .alias(formatcp!("_{SIGNAL}.{MIN}")),
    //     // col(MASS_SPECTRUM)
    //     //     .list()
    //     //     .agg(element().struct_().field_by_name(SIGNAL).max())
    //     //     .alias(formatcp!("_{SIGNAL}.{MAX}")),
    //     // col(MASS_SPECTRUM)
    //     //     .list()
    //     //     .agg(element().struct_().field_by_name(SIGNAL).sum())
    //     //     .alias(formatcp!("_{SIGNAL}.{SUM}")),
    //     // Find
    //     col(MASS_SPECTRUM)
    //         .list()
    //         .agg(as_struct(vec![
    //             element()
    //                 .filter((lit(1) - element().struct_().field_by_name(SIGNAL)).lt(EPSILON))
    //                 .first(),
    //         ]))
    //         .alias(formatcp!("{MASS_SPECTRUM}.Base")),
    //     // Check
    //     mass_spectrum,
    // ]))
}

fn ion(n: u64) -> Expr {
    element()
        .struct_()
        .field_by_name(SIGNAL)
        .filter(
            (element().struct_().field_by_name(MASS_TO_CHARGE) - lit(n))
                .abs()
                .lt(0.5),
        )
        .max()
}

fn rolling(lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    let options = RollingOptionsFixedWindow {
        window_size: key.rolling.window_size,
        min_periods: key.rolling.min_periods,
        center: true,
        ..Default::default()
    };
    let x = col(RETENTION_TIME);
    let y = col(META)
        .struct_()
        .field_by_name(formatcp!("{SIGNAL}.{SUM}"));
    let x_mean = x.clone().rolling_mean(options.clone());
    let y_mean = y.clone().rolling_mean(options.clone());
    let y_median = y.clone().rolling_median(options.clone());
    let xy_mean = (x.clone() * y.clone()).rolling_mean(options.clone());
    let x_std = x.rolling_std(options.clone());
    let y_std = y.rolling_std(options.clone());
    let xy_covariance = xy_mean.clone() - x_mean.clone() * y_mean.clone();
    let xy_correlation = xy_covariance.clone() / (x_std.clone() * y_std.clone());
    let xy_slope = xy_correlation.clone() * (y_std.clone() / x_std.clone());
    let xy_intercept = y_mean.clone() - xy_slope.clone() * x_mean.clone();
    lazy_frame.with_column(col(META).struct_().with_fields(vec![
        x_mean.alias(formatcp!("{ROLLING}.{RETENTION_TIME}.{MEAN}")),
        y_mean.alias(formatcp!("{ROLLING}.{SIGNAL}.{MEAN}")),
        y_median.alias(formatcp!("{ROLLING}.{SIGNAL}.{MEDIAN}")),
        xy_mean.alias(formatcp!("{ROLLING}.{MEAN}")),
        x_std.alias(formatcp!("{ROLLING}.{RETENTION_TIME}.{STANDARD_DEVIATION}")),
        y_std.alias(formatcp!("{ROLLING}.{SIGNAL}.{STANDARD_DEVIATION}")),
        xy_covariance.alias(formatcp!("{ROLLING}.{COVARIANCE}")),
        xy_correlation.alias(formatcp!("{ROLLING}.{CORRELATION}")),
        xy_slope.alias(formatcp!("{ROLLING}.{SLOPE}")),
        xy_intercept.alias(formatcp!("{ROLLING}.{INTERCEPT}")),
    ]))
    // .with_columns([
    //     (x.clone() * y.clone())
    //         .rolling_mean(options.clone())
    //         .alias(formatcp!("_xy.{ROLLING}.{MEAN}")),
    //     x.clone()
    //         .rolling_mean(options.clone())
    //         .alias(formatcp!("_x.{ROLLING}.{MEAN}")),
    //     y.clone()
    //         .rolling_mean(options.clone())
    //         .alias(formatcp!("_y.{ROLLING}.{MEAN}")),
    //     x.rolling_std(options.clone())
    //         .alias(formatcp!("_x.{ROLLING}.{STANDARD_DEVIATION}")),
    //     y.rolling_std(options.clone())
    //         .alias(formatcp!("_y.{ROLLING}.{STANDARD_DEVIATION}")),
    // ])
    // .with_column(
    //     (col(formatcp!("_xy.{ROLLING}.{MEAN}"))
    //         - col(formatcp!("_x.{ROLLING}.{MEAN}")) * col(formatcp!("_y.{ROLLING}.{MEAN}")))
    //     .alias("_xy.cov"),
    // )
    // .with_column(
    //     (col("_xy.cov")
    //         / (col(formatcp!("_x.{ROLLING}.{STANDARD_DEVIATION}"))
    //             * col(formatcp!("_y.{ROLLING}.{STANDARD_DEVIATION}"))))
    //     .alias("Correlation"),
    // )
    // .with_column(
    //     (col("Correlation")
    //         * (col(formatcp!("_y.{ROLLING}.{STANDARD_DEVIATION}"))
    //             / col(formatcp!("_x.{ROLLING}.{STANDARD_DEVIATION}"))))
    //     .alias("Slope"),
    // )
    // .with_column(
    //     (col(formatcp!("_y.{ROLLING}.{MEAN}"))
    //         - col("Slope") * col(formatcp!("_x.{ROLLING}.{MEAN}")))
    //     .alias("Intercept"),
    // )
}

fn threshold(mut lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    println!("lazy_frame TH0: {}", lazy_frame.clone().collect().unwrap());
    // if key.threshold.manual {
    //     let expr = col(MASS_SPECTRUM).list().agg(
    //         // element()
    //         //     .struct_()
    //         //     .field_by_name(MASS_TO_CHARGE)
    //         //     .eq(28.1)
    //         //     .and(element().struct_().field_by_name(SIGNAL).eq(1.0))
    //         //     .any(true),
    //         (element().struct_().field_by_name(MASS_TO_CHARGE) - lit(28))
    //             .abs()
    //             .lt(0.5)
    //             .any(true),
    //     );
    //     lazy_frame =
    //         lazy_frame.with_column(col(META).struct_().with_fields(vec![expr.alias(THRESHOLD)]));
    //     println!("lazy_frame TH1: {}", lazy_frame.clone().collect().unwrap());
    // }
    let mut threshold = lit(true);
    let sum = || {
        col(META)
            .struct_()
            .field_by_name(formatcp!("{SIGNAL}.{SUM}"))
    };
    // Peak max
    if key.threshold.peak_max {
        threshold = threshold.and(sum().peak_max());
    }
    threshold = threshold.and(sum().gt(key.threshold.factor.0));
    lazy_frame = lazy_frame.with_column(
        col(META)
            .struct_()
            .with_fields(vec![threshold.alias(THRESHOLD)]),
    );
    if key.threshold.retention_time != 0.0 {
        // threshold = threshold.and(sum().gt(key.threshold.factor.0));
        let index = (col(RETENTION_TIME) - lit(key.threshold.retention_time.0 * MINUTES))
            .abs()
            .arg_min();
        let t = as_struct(vec![
            col(MASS_SPECTRUM).alias("SOURCE"),
            col(MASS_SPECTRUM).get(index.clone()).alias("TAGRET"),
        ])
        .apply(
            |column| {
                let r#struct = column.struct_()?;
                let source = r#struct.field_by_name("SOURCE")?;
                let tagret = r#struct.field_by_name("TAGRET")?;
                // zip(source.list()?, tagret.list()?)
                //     .map(|(source, tagret)| {
                //         source?.;
                //         Some
                //     })
                //     .collect();
                // let builder = ListPrimitiveChunkedBuilder::new(name, capacity, values_capacity, inner_type)
                for (source, tagret) in zip(source.list()?, tagret.list()?) {
                    let source = source.ok_or(polars_err!(NoData: "SOURCE"))?;
                    let r#struct = source.struct_()?;
                    let mass_to_charge_series = r#struct.field_by_name(MASS_TO_CHARGE)?;
                    let mass_to_charge = mass_to_charge_series.f64()?;
                    let signal_series = r#struct.field_by_name(SIGNAL)?;
                    let signal = signal_series.f64()?;
                    let tagret = tagret.ok_or(polars_err!(NoData: "TAGRET"))?;
                    let tagret = tagret.f64()?;
                    println!("source: {:?}", source);
                    println!("tagret: {:?}", tagret);
                    // cosine(source, point2);
                }
                //
                Ok(column)
            },
            |_, field| Ok(field.clone()),
        );
        lazy_frame = lazy_frame.with_columns([index.clone().alias("index"), t.alias("ms")]);
        println!(
            "!!!!!!!!!!!!!!!!t: {}",
            lazy_frame.clone().collect().unwrap()
        );
    }
    lazy_frame
}

fn cosine_distance(a: Expr, b: Expr) -> Expr {
    lit(1) - (a.clone() * b.clone()).sum() / (a.pow(2).sum().sqrt() * b.pow(2).sum().sqrt())
}

/// Filter and sort threshold
fn filter_and_sort(mut lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    if key.threshold.filter {
        lazy_frame = lazy_frame.filter(col(META).struct_().field_by_name(THRESHOLD));
    } else if key.threshold.sort {
        lazy_frame = lazy_frame.sort_by_exprs(
            [col(META).struct_().field_by_name(THRESHOLD)],
            SortMultipleOptions::default()
                .with_maintain_order(true)
                .with_order_reversed(),
        );
    }
    if key.threshold.manual {
        lazy_frame = lazy_frame.with_column(
            col(MASS_SPECTRUM).list().eval(
                element().filter(
                    element()
                        .struct_()
                        .field_by_name(SIGNAL)
                        .gt(key.threshold.factor.0),
                ),
            ),
        );
    }
    lazy_frame
}

/// Format
fn format(lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    lazy_frame.with_columns([
        // Retention time
        col(RETENTION_TIME)
            .cast(DataType::Duration(TimeUnit::Milliseconds))
            .to_physical()
            / lit(MINUTES),
        // Mass spectrum
        col(MASS_SPECTRUM)
            .list()
            .eval(element().struct_().with_fields(vec![
                    element()
                        .struct_()
                        .field_by_name(MASS_TO_CHARGE)
                        .precision(key.precision, key.significant),
                    element()
                        .struct_()
                        .field_by_name(SIGNAL)
                        .precision(key.precision, key.significant),
                ])),
        // Meta
        col(META).struct_().with_fields(vec![
            col(META)
                .struct_()
                .field_by_names([
                    formatcp!(r#"^{MASS_TO_CHARGE}.*$"#),
                    formatcp!(r#"^{SIGNAL}.*$"#),
                    formatcp!(r#"^{ROLLING}.*$"#),
                ])
                .precision(key.precision, key.significant),
        ]),
    ])
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
