use crate::app::panes::settings::Settings;
use egui::{Direction, Layout, Response, RichText, Ui, Widget};
use egui_extras::{Column, TableBuilder};
use egui_phosphor::regular::LIST;
use polars::prelude::*;
use polars_utils::format_list_truncated;

use super::signal::SignalWidget;

/// Mass spectrum widget
pub struct MassSpectrum<'a> {
    pub(crate) data_frame: &'a DataFrame,
    pub(crate) row_index: usize,
    pub(crate) settings: &'a Settings,
}

impl Widget for MassSpectrum<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let height = ui.spacing().interact_size.y;
        let width = ui.spacing().interact_size.x;
        let mass_spectrum = self.data_frame["MassSpectrum"].list().unwrap();
        let mass_spectrum_series = mass_spectrum.get_as_series(self.row_index).unwrap();
        ui.horizontal(|ui| {
            ui.label(format_list_truncated!(mass_spectrum_series.iter(), 2))
                .on_hover_ui(|ui| {
                    if let Ok(value) = &self.data_frame["MassSpectrum.Count"].get(self.row_index) {
                        ui.label(format!("Count: {value}"));
                    }
                })
                .on_hover_ui(|ui| {
                    ui.heading("Mass to charge");
                    if let Ok(value) = &self.data_frame["MassToCharge.Min"].get(self.row_index) {
                        ui.label(format!("Min: {value}"));
                    }
                    if let Ok(value) = &self.data_frame["MassToCharge.Max"].get(self.row_index) {
                        ui.label(format!("Max: {value}"));
                    }
                })
                .on_hover_ui(|ui| {
                    ui.heading("Signal");
                    if let Ok(value) = &self.data_frame["Signal.Min"].get(self.row_index) {
                        ui.label(format!("Min: {value}"));
                    }
                    if let Ok(value) = &self.data_frame["Signal.Max"].get(self.row_index) {
                        ui.label(format!("Max: {value}"));
                    }
                    if let Ok(value) = &self.data_frame["Signal.Sum"].get(self.row_index) {
                        ui.label(format!("Sum: {value}"));
                    }
                });
            let mut space = ui.available_width();
            if ui.available_width() > height {
                space -= ui.spacing().button_padding.x + height;
            }
            ui.add_space(space);
            ui.visuals_mut().button_frame = false;
            ui.menu_button(RichText::new(LIST), |ui| {
                let total_rows = mass_spectrum_series.len();
                let mass_to_charge_signal = mass_spectrum_series.struct_().unwrap();
                let mass_to_charge_series =
                    mass_to_charge_signal.field_by_name("MassToCharge").unwrap();
                let signal_series = mass_to_charge_signal.field_by_name("Signal").unwrap();
                TableBuilder::new(ui)
                    .cell_layout(Layout::centered_and_justified(Direction::LeftToRight))
                    .column(Column::auto_with_initial_suggestion(width))
                    .columns(Column::auto(), 2)
                    .auto_shrink([true, true])
                    .striped(true)
                    .header(height, |mut row| {
                        row.col(|ui| {
                            ui.heading("Index");
                        });
                        row.col(|ui| {
                            ui.heading("Mass to charge");
                        });
                        row.col(|ui| {
                            ui.heading("Signal");
                        });
                    })
                    .body(|body| {
                        body.rows(height, total_rows, |mut row| {
                            let row_index = row.index();
                            // Index
                            row.col(|ui| {
                                ui.label(row_index.to_string());
                            });
                            // Mass to charge
                            row.col(|ui| {
                                let mass_to_charge = mass_to_charge_series.f32().unwrap();
                                let value = mass_to_charge.get(row_index).unwrap();
                                let formated = self.settings.mass_to_charge.format(value);
                                ui.label(formated).on_hover_text(formated.precision(None));
                            });
                            // Signal
                            row.col(|ui| {
                                // let signal = signal_series.cast(&DataType::Float64).unwrap();
                                // let signal = signal.f64().unwrap();
                                // ui.label(signal.get(row_index).unwrap().to_string());
                                if self.settings.signal.normalize {
                                    let signal = signal_series.f64().unwrap();
                                    ui.add(
                                        SignalWidget::new(signal.get(row_index))
                                            .precision(Some(self.settings.signal.precision)),
                                    );
                                } else {
                                    let signal = signal_series.u16().unwrap();
                                    ui.add(
                                        SignalWidget::new(signal.get(row_index))
                                            .precision(Some(self.settings.signal.precision)),
                                    );
                                };
                            });
                        });
                    });
            });
        })
        .response
    }
}
