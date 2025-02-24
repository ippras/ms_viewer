use crate::app::MAX_PRECISION;
use egui::{ComboBox, DragValue, Grid, Response, Ui, Widget, WidgetText, emath::Float};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
};
use uom::si::{
    f32::Time,
    time::{Units, millisecond, minute, second},
};

/// Settings
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) explode: bool,
    pub(crate) filter_null: bool,
    pub(crate) mass_to_charge: MassToCharge,
    pub(crate) retention_time: RetentionTime,
    pub(crate) signal: Signal,
    pub(crate) peak_max: [bool; 2],
    pub(crate) peak_min: [bool; 2],
    /// The length of the window.
    pub(crate) window_size: usize,
    /// Amount of elements in the window that should be filled before computing a result.
    pub(crate) min_periods: usize,

    pub(crate) sort: Sort,
    pub(crate) plot: Plot,

    pub(crate) visible: Option<bool>,
}

impl Settings {
    fn new() -> Self {
        Self {
            explode: false,
            filter_null: false,
            mass_to_charge: MassToCharge::default(),
            retention_time: RetentionTime::default(),
            signal: Signal::default(),
            peak_max: [false; 2],
            peak_min: [false; 2],
            window_size: 3,
            min_periods: 1,
            sort: Sort::default(),
            plot: Plot::new(),
            visible: None,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

impl Settings {
    pub(crate) fn ui(&mut self, ui: &mut Ui) {
        Grid::new(ui.next_auto_id()).show(ui, |ui| {
            // Retention time
            ui.label("Retention time");
            ComboBox::from_id_salt("RetentionTimeUnits")
                .selected_text(self.retention_time.units.singular())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.retention_time.units,
                        TimeUnits::Millisecond,
                        TimeUnits::Millisecond.singular(),
                    )
                    .on_hover_text(TimeUnits::Millisecond.abbreviation());
                    ui.selectable_value(
                        &mut self.retention_time.units,
                        TimeUnits::Second,
                        TimeUnits::Second.singular(),
                    )
                    .on_hover_text(TimeUnits::Second.abbreviation());
                    ui.selectable_value(
                        &mut self.retention_time.units,
                        TimeUnits::Minute,
                        TimeUnits::Minute.singular(),
                    )
                    .on_hover_text(TimeUnits::Minute.abbreviation());
                })
                .response
                .on_hover_text(format!(
                    "Retention time units {}",
                    self.retention_time.units.abbreviation(),
                ));
            ui.add(DragValue::new(&mut self.retention_time.precision).range(0..=MAX_PRECISION))
                .on_hover_text("Retention time precision");
            ui.end_row();

            // Mass to charge
            ui.label("Mass to charge");
            ui.label("");
            ui.add(DragValue::new(&mut self.mass_to_charge.precision).range(0..=MAX_PRECISION))
                .on_hover_text("Mass to charge precision");
            ui.end_row();

            // Signal
            ui.label("Signal");
            ui.checkbox(&mut self.signal.normalize, "Normalize");
            ui.add(DragValue::new(&mut self.signal.precision).range(0..=MAX_PRECISION))
                .on_hover_text("Signal precision");
            ui.end_row();

            ui.label("Explode");
            ui.checkbox(&mut self.explode, "")
                .on_hover_text("Explode lists");
            ui.end_row();

            ui.label("Filter empty/null");
            ui.checkbox(&mut self.filter_null, "")
                .on_hover_text("Filter empty/null retention time");
            ui.end_row();

            self.sort(ui);
            self.peak_max(ui);
            self.window_size(ui);
            self.min_periods(ui);

            self.legend(ui);
            self.stack(ui);
            self.bar_sort(ui);
            self.bar_width(ui);

            // ui.horizontal(|ui| {
            //     ui.selectable_value(&mut self.visible, Some(true), "◉👁");
            //     ui.selectable_value(&mut self.visible, Some(false), "◎👁");
            // });
        });
    }

    /// Sort
    fn sort(&mut self, ui: &mut Ui) {
        ui.label("Sort");
        ComboBox::from_id_salt(ui.next_auto_id())
            .selected_text(self.sort.text())
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.sort,
                    Sort::RetentionTime,
                    Sort::RetentionTime.text(),
                )
                .on_hover_text(Sort::RetentionTime.description());
                ui.selectable_value(
                    &mut self.sort,
                    Sort::MassToCharge,
                    Sort::MassToCharge.text(),
                )
                .on_hover_text(Sort::MassToCharge.description());
            })
            .response
            .on_hover_text(self.sort.description());
        ui.end_row();
    }

    /// Peak min max
    fn peak_max(&mut self, ui: &mut Ui) {
        ui.label("PeakMinMax");
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.peak_min[0], "");
            ui.checkbox(&mut self.peak_min[1], "Min");
            ui.checkbox(&mut self.peak_max[0], "");
            ui.checkbox(&mut self.peak_max[1], "Max");
        });
        ui.end_row();
    }

    /// Window size
    fn window_size(&mut self, ui: &mut Ui) {
        ui.label("WindowSize");
        ui.add(
            DragValue::new(&mut self.window_size)
                .range(self.min_periods..=usize::MAX)
                .update_while_editing(false),
        )
        .on_hover_text("Window size");
        ui.end_row();
    }

    /// Min periods
    fn min_periods(&mut self, ui: &mut Ui) {
        ui.label("MinPeriods");
        ui.add(
            DragValue::new(&mut self.min_periods)
                .range(1..=self.window_size)
                .update_while_editing(false),
        )
        .on_hover_text("Min periods");
        ui.end_row();
    }

    /// Legend
    fn legend(&mut self, ui: &mut Ui) {
        ui.label("Legend");
        ui.checkbox(&mut self.plot.legend, "")
            .on_hover_text("Show plot legend");
        ui.end_row();
    }

    /// Bar width
    fn bar_width(&mut self, ui: &mut Ui) {
        ui.label("BarWidth");
        ui.add(DragValue::new(&mut self.plot.bar_width).range(0.0..=f64::MAX))
            .on_hover_text("BarWidth.hover");
        ui.end_row();
    }

    /// Bar sort
    fn bar_sort(&mut self, ui: &mut Ui) {
        ui.label("BarSort");
        ComboBox::from_id_salt("BarSort")
            .selected_text(self.plot.bar_sort.text())
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.plot.bar_sort,
                    BarSort::MassToCharge,
                    BarSort::MassToCharge.text(),
                )
                .on_hover_text(BarSort::MassToCharge.description());
                ui.selectable_value(
                    &mut self.plot.bar_sort,
                    BarSort::Signal,
                    BarSort::Signal.text(),
                )
                .on_hover_text(Sort::RetentionTime.description());
            })
            .response
            .on_hover_text(self.plot.bar_sort.description());
        ui.end_row();
    }

    /// Stack
    fn stack(&mut self, ui: &mut Ui) {
        ui.label("Stack");
        ui.checkbox(&mut self.plot.stack, "")
            .on_hover_text("Stack.hover");
        ui.end_row();
    }
}

/// Plot settings
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) struct Plot {
    pub(crate) bar_sort: BarSort,
    pub(crate) bar_width: f64,
    pub(crate) legend: bool,
    pub(crate) stack: bool,
}

impl Plot {
    fn new() -> Self {
        Self {
            bar_sort: BarSort::MassToCharge,
            bar_width: 0.05,
            legend: true,
            stack: false,
        }
    }
}

impl Hash for Plot {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bar_sort.hash(state);
        self.bar_width.ord().hash(state);
        self.legend.hash(state);
        self.stack.hash(state);
    }
}

/// Bar sort
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) enum BarSort {
    #[default]
    MassToCharge,
    Signal,
}

impl BarSort {
    pub(crate) fn text(&self) -> &'static str {
        match self {
            Self::MassToCharge => "MassToCharge",
            Self::Signal => "Signal",
        }
    }

    pub(crate) fn description(&self) -> &'static str {
        match self {
            Self::MassToCharge => "MassToCharge.hover",
            Self::Signal => "Signal.hover",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) enum Sort {
    #[default]
    RetentionTime,
    MassToCharge,
}

impl Sort {
    pub(crate) fn text(&self) -> &'static str {
        match self {
            Self::RetentionTime => "Retention time",
            Self::MassToCharge => "Mass to charge",
        }
    }

    pub(crate) fn description(&self) -> &'static str {
        match self {
            Self::RetentionTime => "Sort by retention time column",
            Self::MassToCharge => "Sort by mass to charge column",
        }
    }
}

/// Mass to charge
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct MassToCharge {
    pub(crate) precision: usize,
}

impl Default for MassToCharge {
    fn default() -> Self {
        Self { precision: 1 }
    }
}

impl MassToCharge {
    pub(crate) fn format(self, value: f32) -> MassToChargeFormat {
        MassToChargeFormat {
            value,
            precision: Some(self.precision),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct MassToChargeFormat {
    value: f32,
    precision: Option<usize>,
}

impl MassToChargeFormat {
    pub(crate) fn precision(self, precision: Option<usize>) -> Self {
        Self { precision, ..self }
    }
}

impl Display for MassToChargeFormat {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let value = self.value;
        if let Some(precision) = self.precision {
            write!(f, "{value:.precision$}")
        } else {
            write!(f, "{value}")
        }
    }
}

impl From<MassToChargeFormat> for WidgetText {
    fn from(value: MassToChargeFormat) -> Self {
        value.to_string().into()
    }
}

/// Retention time
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct RetentionTime {
    pub(crate) precision: usize,
    pub(crate) units: TimeUnits,
}

impl RetentionTime {
    pub(crate) fn format(self, value: i32) -> RetentionTimeFormat {
        RetentionTimeFormat {
            value,
            precision: Some(self.precision),
            units: self.units,
        }
    }
}

impl Default for RetentionTime {
    fn default() -> Self {
        Self {
            precision: 2,
            units: Default::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct RetentionTimeFormat {
    value: i32,
    precision: Option<usize>,
    units: TimeUnits,
}

impl RetentionTimeFormat {
    pub(crate) fn precision(self, precision: Option<usize>) -> Self {
        Self { precision, ..self }
    }
}

impl Display for RetentionTimeFormat {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let milliseconds = || Time::new::<millisecond>(self.value as _);
        let value = match self.units {
            TimeUnits::Millisecond => return write!(f, "{}", self.value),
            TimeUnits::Second => milliseconds().get::<second>(),
            TimeUnits::Minute => milliseconds().get::<minute>(),
        };
        if let Some(precision) = self.precision {
            write!(f, "{value:.precision$}")
        } else {
            write!(f, "{value}")
        }
    }
}

impl From<RetentionTimeFormat> for WidgetText {
    fn from(value: RetentionTimeFormat) -> Self {
        value.to_string().into()
    }
}

/// Time units
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum TimeUnits {
    Millisecond,
    #[default]
    Second,
    Minute,
}

impl TimeUnits {
    pub fn abbreviation(&self) -> &'static str {
        Units::from(*self).abbreviation()
    }

    pub fn singular(&self) -> &'static str {
        Units::from(*self).singular()
    }

    pub fn plural(&self) -> &'static str {
        Units::from(*self).plural()
    }
}

impl From<TimeUnits> for Units {
    fn from(value: TimeUnits) -> Self {
        match value {
            TimeUnits::Millisecond => Units::millisecond(millisecond),
            TimeUnits::Second => Units::second(second),
            TimeUnits::Minute => Units::minute(minute),
        }
    }
}

/// Signal
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Signal {
    pub(crate) normalize: bool,
    pub(crate) precision: usize,
}

impl Default for Signal {
    fn default() -> Self {
        Self {
            normalize: false,
            precision: 2,
        }
    }
}
