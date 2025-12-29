use crate::{
    app::states::pane::settings::{
        Plot, Settings, Sort, Threshold, mass_spectrum::Sort as MassSpectrumSort,
    },
    r#const::*,
    utils::hash::HashedDataFrame,
};
use const_format::formatcp;
use egui::{
    Color32,
    emath::{Float, OrderedFloat},
    util::cache::{ComputerMut, FrameCache},
};
use egui_ext::color;
use egui_plot::Bar;
use indexmap::IndexMap;
use polars::prelude::*;
use std::{collections::HashMap, iter::zip};
// use uom::si::{
//     f64::Time,
//     time::{millisecond, minute, second},
// };

/// Plot computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Plot computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key<'_>) -> PolarsResult<Value> {
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        println!("lazy_frame P0: {}", lazy_frame.clone().collect().unwrap());
        // Compute
        let data_frame = lazy_frame.collect()?;
        // println!("data_frame: {:?}", data_frame.schema());
        let value = compute(&data_frame, key)?;
        Ok(value)
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key<'_>) -> Value {
        self.try_compute(key).expect("Compute plot")
    }
}

/// Plot key
#[derive(Clone, Copy, Hash, Debug)]
pub struct Key<'a> {
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) mass_spectrum_sort: MassSpectrumSort,
    pub(crate) normalize_signal: bool,
    pub(crate) sort: Sort,
    pub(crate) plot: Plot,
    pub(crate) threshold: Threshold,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &Settings) -> Self {
        Self {
            frame,
            mass_spectrum_sort: settings.mass_spectrum.sort,
            normalize_signal: settings.signal.normalize,
            sort: settings.sort,
            plot: settings.plot,
            threshold: settings.threshold,
        }
    }
}

/// Plot value
#[derive(Clone, Debug, Default)]
pub(crate) struct Value {
    pub(crate) bars: IndexMap<OrderedFloat<f64>, Vec<Bar>>,
    pub(crate) thresholds: IndexMap<OrderedFloat<f64>, bool>,
    pub(crate) mass_spectrums: IndexMap<OrderedFloat<f64>, Vec<(f64, f64)>>,
    pub(crate) mean: Option<OrderedFloat<f64>>,
    pub(crate) median: Option<OrderedFloat<f64>>,
    pub(crate) rolling_mean: Vec<[f64; 2]>,
    pub(crate) rolling_median: Vec<[f64; 2]>,
}

fn compute(data_frame: &DataFrame, key: Key) -> PolarsResult<Value> {
    match key.sort {
        Sort::RetentionTime => by_retention_time(data_frame, key),
        Sort::MassToCharge => by_mass_to_charge(data_frame, key),
    }
}

// RETENTION_TIME: Vec<Bar>, stacked, sorted by MASS_TO_CHARGE
fn by_retention_time(data_frame: &DataFrame, key: Key) -> PolarsResult<Value> {
    let meta = data_frame[META].struct_()?;
    let threshold_series = meta.field_by_name(THRESHOLD)?;
    let threshold = threshold_series.bool()?;
    let retention_time = if key.threshold.filter {
        &data_frame[RETENTION_TIME].f64()?.filter(threshold)?
    } else {
        data_frame[RETENTION_TIME].f64()?
    };
    let mass_spectrum = if key.threshold.filter {
        &data_frame[MASS_SPECTRUM].list()?.filter(threshold)?
    } else {
        data_frame[MASS_SPECTRUM].list()?
    };
    let mut value = Value::default();
    let mut offsets = HashMap::new();
    for ((retention_time, mass_spectrum), threshold) in
        zip(zip(retention_time, mass_spectrum), threshold)
    {
        let Some(retention_time) = retention_time else {
            polars_bail!(NoData: "{RETENTION_TIME}");
        };
        let Some(mass_spectrum) = mass_spectrum else {
            polars_bail!(NoData: "{MASS_SPECTRUM}");
        };
        let Some(threshold) = threshold else {
            polars_bail!(NoData: "{THRESHOLD}");
        };
        value
            .thresholds
            .entry(retention_time.ord())
            .or_insert(threshold);
        let mass_spectrum = mass_spectrum.struct_()?;
        // MASS_SPECTRUM: MASS_TO_CHARGE | SIGNAL
        for (mass_to_charge, signal) in zip(
            mass_spectrum.field_by_name(MASS_TO_CHARGE)?.f64()?,
            mass_spectrum.field_by_name(SIGNAL)?.f64()?,
        ) {
            let mass_to_charge = mass_to_charge.unwrap_or_default();
            let signal = signal.unwrap_or_default();
            value
                .mass_spectrums
                .entry(retention_time.ord())
                .or_insert_with(Vec::new)
                .push((mass_to_charge, signal));
            let signal = signal as _;
            let offset = offsets.entry(retention_time.ord()).or_default();
            let mut bar = Bar::new(retention_time, signal)
                .name(mass_to_charge.to_string())
                .width(key.plot.width.0);
            if key.plot.fill {
                bar = bar.fill(color(mass_to_charge.round() as usize));
            }
            // .fill(Color32::RED.lerp_to_gamma(Color32::BLUE, mass_to_charge as f32 % 10.0))
            // .stroke((1.0, color(mass_to_charge.round() as usize)));
            if key.plot.stack {
                bar = bar.base_offset(*offset);
            }
            *offset += signal;
            value
                .bars
                .entry(mass_to_charge.ord())
                .or_insert_with(Vec::new)
                .push(bar);
        }
    }
    if key.plot.stack {
        let retention_time = data_frame[RETENTION_TIME].f64()?;
        let rolling_mean_series = meta.field_by_name(formatcp!("{ROLLING}.{SIGNAL}.{MEAN}"))?;
        let rolling_median_series = meta.field_by_name(formatcp!("{ROLLING}.{SIGNAL}.{MEDIAN}"))?;
        let rolling_mean = rolling_mean_series.f64()?;
        let rolling_median = rolling_median_series.f64()?;
        let sum_series = meta.field_by_name(formatcp!("{SIGNAL}.{SUM}"))?;
        let sum = sum_series.f64()?;
        value.mean = sum.mean().map(Float::ord);
        value.median = sum.median().map(Float::ord);
        for (retention_time, (rolling_mean, rolling_median)) in
            zip(retention_time, zip(rolling_mean, rolling_median))
        {
            let Some(retention_time) = retention_time else {
                polars_bail!(NoData: "{RETENTION_TIME}");
            };
            let Some(rolling_mean) = rolling_mean else {
                continue;
            };
            let Some(rolling_median) = rolling_median else {
                continue;
            };
            value.rolling_mean.push([retention_time, rolling_mean]);
            value.rolling_median.push([retention_time, rolling_median]);
        }
    }
    Ok(value)
}

fn by_mass_to_charge(data_frame: &DataFrame, key: Key) -> PolarsResult<Value> {
    unreachable!()
}
