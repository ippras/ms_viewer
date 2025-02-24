use super::signal::SignalWidget;
use crate::app::states::settings::Settings;
use egui::{Direction, Layout, Response, RichText, Ui, Widget};
use egui_extras::{Column, TableBuilder};
use egui_phosphor::regular::LIST;
use polars::prelude::*;
use polars_utils::format_list_truncated;

// https://en.wikipedia.org/wiki/Mass_chromatogram

/// Extracted ion chromatogram (EIC or XIC) widget
pub struct IonChromatogram<'a> {
    pub(crate) data_frame: &'a DataFrame,
    pub(crate) row_index: usize,
    pub(crate) settings: &'a Settings,
}

impl IonChromatogram<'_> {
    fn show(&self, ui: &mut Ui) -> PolarsResult<Response> {
        let height = ui.spacing().interact_size.y;
        let width = ui.spacing().interact_size.x;
        let ion_chromatogram = self.data_frame["ExtractedIonChromatogram"].list().unwrap();
        let ion_chromatogram_series = ion_chromatogram.get_as_series(self.row_index).unwrap();
        let t = ion_chromatogram_series.iter();
        let response = ui
            .horizontal(|ui| {
                ui.label(format_list_truncated!(t, 2))
                    .on_hover_ui(|ui| {
                        if let Ok(count) =
                            &self.data_frame["ExtractedIonChromatogram.Count"].get(self.row_index)
                        {
                            ui.label(format!("Count: {count}"));
                        }
                    })
                    .on_hover_ui(|ui| {
                        ui.heading("RetentionTime");
                        if let Ok(min) = &self.data_frame["RetentionTime.Min"].get(self.row_index) {
                            ui.label(format!("Min: {min}"));
                        }
                        if let Ok(max) = &self.data_frame["RetentionTime.Max"].get(self.row_index) {
                            ui.label(format!("Max: {max}"));
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
                    let total_rows = ion_chromatogram_series.len();
                    let retention_time_signal = ion_chromatogram_series.struct_().unwrap();
                    let retention_time_series = retention_time_signal
                        .field_by_name("RetentionTime")
                        .unwrap();
                    let signal_series = retention_time_signal.field_by_name("Signal").unwrap();
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
                                ui.heading("Retention time");
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
                                // Retention time
                                row.col(|ui| {
                                    let retention_time = retention_time_series.i32().unwrap();
                                    let value = retention_time.get(row_index).unwrap();
                                    let formated = self.settings.retention_time.format(value);
                                    ui.label(formated).on_hover_text(formated.precision(None));
                                });
                                // Signal
                                row.col(|ui| {
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
            .response;
        Ok(response)
    }
}

impl Widget for IonChromatogram<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui).expect("show ion chromatogram widget")
    }
}
