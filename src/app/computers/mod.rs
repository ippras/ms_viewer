use crate::{
    app::states::pane::settings::{
        Rolling, Settings, Threshold,
        mass_spectrum::{MassSpectrum, Sort as MassSpectrumSort},
    },
    r#const::*,
    utils::hash::HashedDataFrame,
};
use const_format::formatcp;
use egui::util::cache::{ComputerMut, FrameCache};
use polars::prelude::*;
use std::f64::EPSILON;
use tracing::{instrument, trace};

const MINUTES: f64 = 60_000.0;

/// Table computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Table computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    #[instrument(skip(self), err)]
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
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
        lazy_frame = retention_time(lazy_frame, key);
        lazy_frame = meta(lazy_frame, key)?;
        lazy_frame = rolling(lazy_frame, key);
        lazy_frame = threshold(lazy_frame, key);
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

/// Table key
#[derive(Clone, Copy, Hash, Debug)]
pub struct Key<'a> {
    // pub(crate) sort: Sort,
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) filter_null: bool,
    pub(crate) mass_spectrum: MassSpectrum,
    pub(crate) normalize_signal: bool,
    pub(crate) rolling: Rolling,
    pub(crate) threshold: Threshold,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &Settings) -> Self {
        Self {
            // sort: settings.sort,
            frame,
            filter_null: settings.filter_null,
            mass_spectrum: settings.mass_spectrum,
            normalize_signal: settings.signal.normalize,
            rolling: settings.rolling,
            threshold: settings.threshold,
        }
    }
}

/// Table value
type Value = HashedDataFrame;

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

fn threshold(lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    let sum = || {
        col(META)
            .struct_()
            .field_by_name(formatcp!("{SIGNAL}.{SUM}"))
    };
    // Peak max
    let mut threshold = lit(true);
    if key.threshold.peak_max {
        threshold = threshold.and(sum().peak_max());
    }
    threshold = threshold.and(sum().gt(key.threshold.factor.0));
    lazy_frame.with_column(
        col(META)
            .struct_()
            .with_fields(vec![threshold.alias(THRESHOLD)]),
    )
}

pub(crate) mod filter_and_sort;
pub(crate) mod plot;
pub(crate) mod table;
