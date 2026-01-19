use std::{ops::Deref, slice::Iter, vec::IntoIter};

use self::mass_spectrum::{MassSpectrum, Sort as MassSpectrumSort};
use crate::app::MAX_PRECISION;
use egui::{ComboBox, DragValue, Label, Slider, Ui, Widget};
use egui_dnd::dnd;
use egui_ext::LabeledSeparator;
use egui_l20n::prelude::*;
use egui_phosphor::regular::{CHART_BAR, DOTS_SIX_VERTICAL, MINUS, PLUS, SORT_ASCENDING, TABLE};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

/// Settings
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) percent: bool,
    pub(crate) precision: usize,
    pub(crate) significant: bool,

    pub(crate) explode: bool,
    pub(crate) filter_null: bool,
    pub(crate) mass_to_charge: MassToCharge,
    pub(crate) signal: Signal,

    pub(crate) sort: Sort,
    pub(crate) visible: Option<bool>,

    pub(crate) edit: bool,
    pub(crate) view: View,
    // Rolling
    pub(crate) rolling: Rolling,
    // Plot
    pub(crate) plot: Plot,
    // Table
    pub(crate) table: Table,
    // Threshold
    pub(crate) retention_times: RetentionTimes,
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
            significant: false,
            explode: false,
            filter_null: false,
            mass_to_charge: MassToCharge::default(),
            signal: Signal::default(),
            rolling: Rolling::new(),
            sort: Sort::default(),
            visible: None,

            view: View::default(),
            edit: false,
            plot: Plot::new(),
            table: Table::new(),
            retention_times: RetentionTimes::new(),
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

        // Retention times
        ui.labeled_separator(ui.localize("RetentionTimes"));
        self.retention_times(ui);

        // Plot
        ui.labeled_separator(ui.localize("Plot"));
        self.plot(ui);

        // let mut items = vec!["alfred", "bernhard", "christian"];
        // let mut items = vec![0, 0, 0];
        // let response = dnd(ui, ui.auto_id_with("RetentionTimes")).show_vec(
        //     &mut items,
        //     |ui, item, handle, state| {
        //         ui.horizontal(|ui| {
        //             handle.ui(ui, |ui| {
        //                 ui.label(DOTS_SIX_VERTICAL);
        //             });
        //             ui.push_id(state.index, |ui| {
        //                 ui.label(format!("{item}"));
        //             });
        //         });
        //     },
        // );
        // if response.is_drag_finished() {
        //     response.update_vec(self.0.as_mut_slice());
        // }

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

    /// Retention times
    fn retention_times(&mut self, ui: &mut Ui) {
        self.retention_times.show(ui);
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

    /// Table
    fn table(&mut self, ui: &mut Ui) {
        self.table.show(ui);
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

/// Retention times
#[derive(Clone, Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct RetentionTimes(Vec<i32>);

impl RetentionTimes {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn show(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Retention times");
            if ui.button(PLUS).clicked() {
                self.0.push(0);
            }
            if ui.button(SORT_ASCENDING).clicked() {
                self.0.sort();
            }
        });
        let mut delete = None;
        let response = dnd(ui, ui.auto_id_with("RetentionTimes")).show(
            self.0.iter_mut().enumerate(),
            |ui, (_, item), handle, state| {
                ui.horizontal(|ui| {
                    handle.ui(ui, |ui| {
                        ui.label(DOTS_SIX_VERTICAL);
                    });
                    DragValue::new(item)
                        .range(0..=i32::MAX)
                        .update_while_editing(false)
                        .ui(ui);
                    if ui.button(MINUS).clicked() {
                        delete = Some(state.index);
                    }
                });
                // DragValue::new(item)
                //     .range(0..=i32::MAX)
                //     .update_while_editing(false)
                //     .ui(ui);
                // ui.horizontal(|ui| {
                //     // let visible = index.visible;
                //     ui.push_id(state.index, |ui| {
                //         handle.ui(ui, |ui| {
                //             ui.label(DOTS_SIX_VERTICAL);
                //         });
                //         DragValue::new(item)
                //             .range(0..=i32::MAX)
                //             .update_while_editing(false)
                //             .ui(ui);
                //     });
                //     // ui.checkbox(&mut index.visible, "");
                //     // let mut text = RichText::new(ui.localize(&index.name));
                //     // if !visible {
                //     //     text = text.weak();
                //     // }
                //     // let response = ui.label(text);
                //     // Popup::context_menu(&response)
                //     //     .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
                //     //     .show(|ui| {
                //     //         if ui.button("Show all").clicked() {
                //     //             visible_all = Some(true);
                //     //         }
                //     //         if ui.button("Hide all").clicked() {
                //     //             visible_all = Some(false);
                //     //         }
                //     //     });
                // });
            },
        );
        if let Some(index) = delete {
            self.0.remove(index);
        }
        if response.is_drag_finished() {
            response.update_vec(self.0.as_mut_slice());
        }
    }
}

impl Deref for RetentionTimes {
    type Target = Vec<i32>;

    fn deref(&self) -> &Self::Target {
        &self.0
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

/// Table settings
#[derive(Clone, Copy, Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Table {
    pub(crate) reset: bool,
    pub(crate) resizable: bool,
    pub(crate) sticky_columns: usize,
    pub(crate) truncate_headers: bool,
}

impl Table {
    pub(crate) fn new() -> Self {
        Self {
            reset: false,
            resizable: false,
            sticky_columns: 0,
            truncate_headers: false,
        }
    }

    fn show(&mut self, ui: &mut Ui) {
        self.sticky(ui);
        self.truncate(ui);
    }

    /// Sticky columns
    fn sticky(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("StickyColumns"))
                .on_hover_localized("StickyColumns.hover");
            Slider::new(&mut self.sticky_columns, 0..=8).ui(ui);
        });
    }

    /// Truncate headers
    fn truncate(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("TruncateHeaders"))
                .on_hover_localized("TruncateHeaders.hover");
            ui.checkbox(&mut self.truncate_headers, ());
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
