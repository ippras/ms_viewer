use super::signal::SignalWidget;
use crate::{app::states::pane::settings::Settings, r#const::*};
use const_format::formatcp;
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
        let eic = self.data_frame[EIC].list()?;
        let eic_series = eic.get_as_series(self.row_index).unwrap();
        let response = ui
            .horizontal(|ui| {
                ui.label(format_list_truncated!(eic_series.iter(), 2))
                    .on_hover_ui(|ui| {
                        if let Ok(count) =
                            &self.data_frame[formatcp!("{EIC}.{LEN}")].get(self.row_index)
                        {
                            ui.label(format!("Length: {count}"));
                        }
                    })
                    .on_hover_ui(|ui| {
                        ui.heading(RETENTION_TIME);
                        if let Ok(min) = &self.data_frame[formatcp!("{RETENTION_TIME}.{MIN}")]
                            .get(self.row_index)
                        {
                            ui.label(format!("Min: {min}"));
                        }
                        if let Ok(max) = &self.data_frame[formatcp!("{RETENTION_TIME}.{MAX}")]
                            .get(self.row_index)
                        {
                            ui.label(format!("Max: {max}"));
                        }
                    })
                    .on_hover_ui(|ui| {
                        ui.heading("Signal");
                        if let Ok(value) =
                            &self.data_frame[formatcp!("{SIGNAL}.{MIN}")].get(self.row_index)
                        {
                            ui.label(format!("Min: {value}"));
                        }
                        if let Ok(value) =
                            &self.data_frame[formatcp!("{SIGNAL}.{MAX}")].get(self.row_index)
                        {
                            ui.label(format!("Max: {value}"));
                        }
                        if let Ok(value) =
                            &self.data_frame[formatcp!("{SIGNAL}.{SUM}")].get(self.row_index)
                        {
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
                    let total_rows = eic_series.len();
                    let retention_time_signal = eic_series.struct_().unwrap();
                    let retention_time_series =
                        retention_time_signal.field_by_name(RETENTION_TIME).unwrap();
                    let signal_series = retention_time_signal.field_by_name(SIGNAL).unwrap();
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
                                    // let formated = self.settings.retention_time.format(value);
                                    ui.label(value.to_string());
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
