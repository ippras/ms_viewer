use self::mass_spectrum::{MassSpectrum, Sort as MassSpectrumSort};
use crate::app::MAX_PRECISION;
use egui::{ComboBox, DragValue, Slider, Ui, Widget, WidgetText};
use egui_ext::LabeledSeparator;
use egui_l20n::prelude::*;
use egui_phosphor::regular::{CHART_BAR, TABLE};
use egui_tiles::ContainerKind;
use ordered_float::OrderedFloat;
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
    pub(crate) percent: bool,
    pub(crate) precision: usize,
    pub(crate) resizable: bool,
    pub(crate) significant: bool,

    pub(crate) explode: bool,
    pub(crate) filter_null: bool,
    pub(crate) mass_to_charge: MassToCharge,
    pub(crate) retention_time: RetentionTime,
    pub(crate) signal: Signal,

    pub(crate) sort: Sort,
    pub(crate) visible: Option<bool>,

    pub(crate) edit: bool,
    pub(crate) view: View,
    // Rolling
    pub(crate) rolling: Rolling,
    // Plot
    pub(crate) plot: Plot,
    // Threshold
    pub(crate) threshold: Threshold,
    // Mass spectrum
    pub(crate) mass_spectrum: MassSpectrum,
}

impl Settings {
    pub(crate) fn new() -> Self {
        Self {
            percent: true,
            precision: 1,
            resizable: false,
            significant: false,
            explode: false,
            filter_null: false,
            mass_to_charge: MassToCharge::default(),
            retention_time: RetentionTime::default(),
            signal: Signal::default(),
            rolling: Rolling::new(),
            sort: Sort::default(),
            visible: None,

            view: View::default(),
            edit: false,
            plot: Plot::new(),
            threshold: Threshold::new(),
            mass_spectrum: MassSpectrum::new(),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

impl Settings {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        self.precision(ui);
        self.significant(ui);
        self.percent(ui);

        self.retention_time(ui);
        self.explode(ui);
        self.filter(ui);
        self.signal(ui);

        self.sort(ui);

        // Rolling
        ui.labeled_separator(ui.localize("Rolling"));
        self.rolling(ui);

        // Threshold
        ui.labeled_separator(ui.localize("Threshold"));
        self.threshold(ui);

        // Mass spectrum
        ui.labeled_separator(ui.localize("MassSpectrum"));
        self.mass_spectrum(ui);

        // Plot
        ui.labeled_separator(ui.localize("Plot"));
        self.plot(ui);

        // Grid::new(ui.next_auto_id()).show(ui, |ui| {
        //     // // Mass to charge
        //     // ui.label("Mass to charge");
        //     // ui.label("");
        //     // ui.add(DragValue::new(&mut self.mass_to_charge.precision).range(0..=MAX_PRECISION))
        //     //     .on_hover_text("Mass to charge precision");
        //     // ui.end_row();
        //     // // Signal
        //     // ui.label("Signal");
        //     // ui.checkbox(&mut self.signal.normalize, "Normalize");
        //     // ui.add(DragValue::new(&mut self.signal.precision).range(0..=MAX_PRECISION))
        //     //     .on_hover_text("Signal precision");
        //     // ui.end_row();
        //     // ui.horizontal(|ui| {
        //     //     ui.selectable_value(&mut self.visible, Some(true), "◉👁");
        //     //     ui.selectable_value(&mut self.visible, Some(false), "◎👁");
        //     // });
        // });
    }

    /// Precision
    fn precision(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("Precision"))
                .on_hover_localized("Precision.hover");
            Slider::new(&mut self.precision, 1..=MAX_PRECISION)
                .update_while_editing(false)
                .ui(ui);
        });
    }

    // Significant
    fn significant(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("Significant"))
                .on_hover_localized("Significant.hover");
            ui.checkbox(&mut self.significant, ());
        });
    }

    /// Percent
    fn percent(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("Percent"))
                .on_hover_localized("Percent.hover");
            ui.checkbox(&mut self.percent, ());
        });
    }

    /// Retention time
    fn retention_time(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Retention time");
            ComboBox::from_id_salt("RetentionTime")
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
            DragValue::new(&mut self.retention_time.precision)
                .range(0..=MAX_PRECISION)
                .ui(ui)
                .on_hover_text("Retention time precision");
        });
    }

    /// Explode
    fn explode(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Explode");
            ui.checkbox(&mut self.explode, ())
                .on_hover_text("Explode lists");
        });
    }

    /// Filter
    fn filter(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Filter empty/null");
            ui.checkbox(&mut self.filter_null, ())
                .on_hover_text("Filter empty/null retention time");
        });
    }

    // Signal
    fn signal(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Signal");
            ui.checkbox(&mut self.signal.normalize, "Normalize");
        });
    }

    /// Sort
    fn sort(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Sort");
            ComboBox::from_id_salt(ui.next_auto_id())
                .selected_text(self.sort.text())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.sort,
                        Sort::RetentionTime,
                        Sort::RetentionTime.text(),
                    )
                    .on_hover_text(Sort::RetentionTime.hover_text());
                    ui.selectable_value(
                        &mut self.sort,
                        Sort::MassToCharge,
                        Sort::MassToCharge.text(),
                    )
                    .on_hover_text(Sort::MassToCharge.hover_text());
                })
                .response
                .on_hover_text(self.sort.hover_text());
        });
    }

    /// Rolling
    fn rolling(&mut self, ui: &mut Ui) {
        self.rolling.show(ui);
    }

    /// Thresholded
    fn threshold(&mut self, ui: &mut Ui) {
        self.threshold.show(ui);
    }

    /// Mass spectrum
    fn mass_spectrum(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Sort");
            ComboBox::from_id_salt(ui.next_auto_id())
                .selected_text(self.mass_spectrum.sort.text())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.mass_spectrum.sort,
                        MassSpectrumSort::MassToCharge,
                        MassSpectrumSort::MassToCharge.text(),
                    )
                    .on_hover_text(MassSpectrumSort::MassToCharge.hover_text());
                    ui.selectable_value(
                        &mut self.mass_spectrum.sort,
                        MassSpectrumSort::Signal,
                        MassSpectrumSort::Signal.text(),
                    )
                    .on_hover_text(MassSpectrumSort::Signal.hover_text());
                })
                .response
                .on_hover_text(self.mass_spectrum.sort.hover_text());
        });
    }

    /// Plot
    fn plot(&mut self, ui: &mut Ui) {
        self.plot.show(ui);
    }
}

/// Sort
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

    pub(crate) fn hover_text(&self) -> &'static str {
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

/// Rolling
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Rolling {
    /// The length of the window.
    pub(crate) window_size: usize,
    /// Amount of elements in the window that should be filled before computing a result.
    pub(crate) min_periods: usize,
}

impl Rolling {
    fn new() -> Self {
        Self {
            window_size: 3,
            min_periods: 1,
        }
    }

    fn show(&mut self, ui: &mut Ui) {
        self.window_size(ui);
        self.min_periods(ui);
    }

    /// Window size
    fn window_size(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("WindowSize").on_hover_text("WindowSize.hover");
            DragValue::new(&mut self.window_size)
                .range(self.min_periods..=usize::MAX)
                .update_while_editing(false)
                .ui(ui);
        });
    }

    /// Min periods
    fn min_periods(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("MinPeriods").on_hover_text("MinPeriods");
            DragValue::new(&mut self.min_periods)
                .range(1..=self.window_size)
                .update_while_editing(false)
                .ui(ui);
        });
    }
}

/// Plot settings
#[derive(Clone, Copy, Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Plot {
    pub(crate) fill: bool,
    pub(crate) legend: bool,
    pub(crate) stack: bool,
    pub(crate) width: OrderedFloat<f64>,
}

impl Plot {
    fn new() -> Self {
        Self {
            fill: false,
            legend: true,
            stack: false,
            width: OrderedFloat(0.05),
        }
    }

    fn show(&mut self, ui: &mut Ui) {
        self.legend(ui);
        self.stack(ui);
        self.fill(ui);
        self.width(ui);
    }

    /// Fill
    fn fill(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Fill").on_hover_text("Fill.hover");
            ui.checkbox(&mut self.fill, ());
        });
    }

    /// Legend
    fn legend(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Legend").on_hover_text("Show plot legend");
            ui.checkbox(&mut self.legend, ());
        });
    }

    /// Width
    fn width(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("BarWidth").on_hover_text("BarWidth.hover");
            DragValue::new(&mut self.width.0)
                .custom_formatter(|n, _range| n.to_string())
                .range(0.0..=f64::MAX)
                .speed(0.001)
                .ui(ui);
        });
    }

    /// Stack
    fn stack(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Stack").on_hover_text("Stack.hover");
            ui.checkbox(&mut self.stack, ());
        });
    }
}

/// Threshold
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Threshold {
    pub(crate) peak_max: bool,
    pub(crate) retention_time: OrderedFloat<f64>,
    pub(crate) factor: OrderedFloat<f64>,
    // pub(crate) manual: bool,
    pub(crate) sort: bool,
    pub(crate) filter: bool,
}

impl Threshold {
    fn new() -> Self {
        Self {
            peak_max: false,
            retention_time: OrderedFloat(0.0),
            factor: OrderedFloat(0.0),
            // manual: false,
            filter: false,
            sort: false,
        }
    }

    fn show(&mut self, ui: &mut Ui) {
        self.peak_max(ui);
        ui.separator();
        self.retention_time(ui);
        self.factor(ui);
        ui.separator();
        // self.manual(ui);
        self.sort(ui);
        self.filter(ui);
    }

    /// Retention time
    fn retention_time(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("Threshold_RetentionTime"))
                .on_hover_localized("Threshold_RetentionTime.hover");
            DragValue::new(&mut self.retention_time.0)
                .custom_formatter(|n, _range| n.to_string())
                .range(0.0..=f64::MAX)
                .update_while_editing(false)
                .ui(ui);
        });
    }

    /// Factor
    fn factor(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("Threshold_Factor"))
                .on_hover_localized("Threshold_Factor.hover");
            DragValue::new(&mut self.factor.0)
                .range(0.0..=1.0)
                .speed(0.01)
                .update_while_editing(false)
                .ui(ui);
        });
    }

    /// Filter
    fn filter(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("Threshold_Filter"))
                .on_hover_localized("Threshold_Filter.hover");
            ui.checkbox(&mut self.filter, ());
        });
    }

    // /// Manual
    // fn manual(&mut self, ui: &mut Ui) {
    //     ui.horizontal(|ui| {
    //         ui.label("Threshold_Manual")
    //             .on_hover_text("Threshold_Manual.hover");
    //         ui.checkbox(&mut self.manual, ());
    //     });
    // }

    /// Peak max
    fn peak_max(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("PeakMax");
            ui.checkbox(&mut self.peak_max, ());
        });
    }

    /// Sort by threshold
    fn sort(&mut self, ui: &mut Ui) {
        ui.add_enabled_ui(!self.filter, |ui| {
            ui.horizontal(|ui| {
                ui.label(ui.localize("Threshold_Sort"))
                    .on_hover_localized("Threshold_Sort.hover");
                ui.checkbox(&mut self.sort, ());
            });
        });
    }
}

/// View
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) enum View {
    Plot,
    #[default]
    Table,
}

impl View {
    pub(crate) fn icon(&self) -> &'static str {
        match self {
            Self::Plot => CHART_BAR,
            Self::Table => TABLE,
        }
    }

    pub(crate) fn text(&self) -> &'static str {
        match self {
            Self::Plot => "Plot",
            Self::Table => "Table",
        }
    }

    pub(crate) fn hover_text(&self) -> &'static str {
        match self {
            Self::Plot => "Plot.hover",
            Self::Table => "Table.hover",
        }
    }
}

pub(crate) mod mass_spectrum;
