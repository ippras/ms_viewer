use crate::{
    app::states::settings::{BarSort, Settings, Sort},
    r#const::*,
    utils::hash::HashedDataFrame,
};
use const_format::formatcp;
use egui::{
    emath::{Float, OrderedFloat},
    util::cache::{ComputerMut, FrameCache},
};
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
        // Sort
        lazy_frame = sort(lazy_frame, key);
        // Compute
        let data_frame = lazy_frame.collect()?;
        println!("data_frame: {:?}", data_frame.schema());
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
    pub(crate) bar_sort: BarSort,
    pub(crate) bar_width: OrderedFloat<f64>,
    pub(crate) normalize_signal: bool,
    pub(crate) peak_max: [bool; 2],
    pub(crate) peak_min: [bool; 2],
    pub(crate) sort: Sort,
    pub(crate) stack: bool,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &Settings) -> Self {
        Self {
            frame,
            bar_sort: settings.plot.bar_sort,
            bar_width: settings.plot.bar_width.ord(),
            normalize_signal: settings.signal.normalize,
            peak_max: settings.peak_max,
            peak_min: settings.peak_min,
            sort: settings.sort,
            stack: settings.plot.stack,
        }
    }
}

/// Plot value
#[derive(Clone, Debug, Default)]
pub(crate) struct Value {
    pub(crate) bars: IndexMap<OrderedFloat<f32>, Vec<Bar>>,
    pub(crate) mass_spectrums: IndexMap<OrderedFloat<f64>, Vec<(f32, f64)>>,
    pub(crate) mean: Option<OrderedFloat<f64>>,
    pub(crate) median: Option<OrderedFloat<f64>>,
    pub(crate) rolling_mean: Vec<[f64; 2]>,
}

/// Sort
fn sort(mut lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    // Sort mass spectrum
    lazy_frame = match key.bar_sort {
        BarSort::MassToCharge => {
            lazy_frame.with_column(col(MASS_SPECTRUM).list().eval(element().sort_by(
                [element().struct_().field_by_name(MASS_TO_CHARGE)],
                SortMultipleOptions::new(),
            )))
        }
        BarSort::Signal => {
            lazy_frame.with_column(col(MASS_SPECTRUM).list().eval(element().sort_by(
                [element().struct_().field_by_name(SIGNAL)],
                SortMultipleOptions::new(),
            )))
        }
    };
    lazy_frame
}

fn compute(data_frame: &DataFrame, key: Key) -> PolarsResult<Value> {
    match key.sort {
        Sort::RetentionTime => by_retention_time(data_frame, key),
        Sort::MassToCharge => by_mass_to_charge(data_frame, key),
    }
}

// RETENTION_TIME: Vec<Bar>, stacked, sorted by MASS_TO_CHARGE
fn by_retention_time(data_frame: &DataFrame, key: Key) -> PolarsResult<Value> {
    let filter = data_frame["_Filter"].bool()?;
    let retention_time = &data_frame[RETENTION_TIME].f64()?.filter(filter)?;
    let mass_spectrum = &data_frame[MASS_SPECTRUM].list()?.filter(filter)?;
    let mut value = Value::default();
    let mut offsets = HashMap::new();
    // RETENTION_TIME | MASS_SPECTRUM
    for (retention_time, mass_spectrum) in zip(retention_time, mass_spectrum) {
        let Some(retention_time) = retention_time else {
            polars_bail!(NoData: "{RETENTION_TIME}");
        };
        let Some(mass_spectrum) = mass_spectrum else {
            polars_bail!(NoData: "{MASS_SPECTRUM}");
        };
        let mass_spectrum = mass_spectrum.struct_()?;
        // MASS_SPECTRUM: MASS_TO_CHARGE | SIGNAL
        for (mass_to_charge, signal) in zip(
            mass_spectrum.field_by_name(MASS_TO_CHARGE)?.f32()?,
            mass_spectrum.field_by_name(SIGNAL)?.f64()?,
        ) {
            let Some(mass_to_charge) = mass_to_charge else {
                polars_bail!(NoData: "{MASS_TO_CHARGE}");
            };
            let Some(signal) = signal else {
                polars_bail!(NoData: "{SIGNAL}");
            };
            value
                .mass_spectrums
                .entry(retention_time.ord())
                .or_insert_with(Vec::new)
                .push((mass_to_charge, signal));
            let signal = signal as _;
            let offset = offsets.entry(retention_time.ord()).or_default();
            let mut bar = Bar::new(retention_time, signal)
                .name(mass_to_charge.to_string())
                .width(key.bar_width.0);
            if key.stack {
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
    if key.stack {
        let retention_time = data_frame[RETENTION_TIME].f64()?;
        let rolling_mean = data_frame[formatcp!("_y.{ROLLING}.{MEAN}")].f64()?;
        let sum = &data_frame[formatcp!("_{SIGNAL}.{SUM}")];
        value.mean = sum.f64()?.mean().map(Float::ord);
        value.median = sum.f64()?.median().map(Float::ord);
        for (retention_time, rolling_mean) in zip(retention_time, rolling_mean) {
            let Some(retention_time) = retention_time else {
                polars_bail!(NoData: "{RETENTION_TIME}");
            };
            let Some(rolling_mean) = rolling_mean else {
                continue;
            };
            value.rolling_mean.push([retention_time, rolling_mean]);
        }
    }
    Ok(value)
}

fn by_mass_to_charge(data_frame: &DataFrame, key: Key) -> PolarsResult<Value> {
    unreachable!()
}
