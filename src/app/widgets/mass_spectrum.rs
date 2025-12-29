use crate::{app::states::pane::settings::Settings, r#const::*};
use const_format::formatcp;
use egui::{Direction, Layout, Response, RichText, Ui, Widget, WidgetText};
use egui_ext::ResponseExt;
use egui_extras::{Column, TableBuilder, TableRow};
use egui_phosphor::regular::LIST;
use polars::prelude::*;
use polars_utils::format_list_truncated;
use tracing::instrument;

/// Mass spectrum widget
pub struct MassSpectrum<'a> {
    pub(crate) data_frame: &'a DataFrame,
    pub(crate) index: usize,
    pub(crate) settings: &'a Settings,
}

impl MassSpectrum<'_> {
    // pub fn show(self, ui: &mut Ui) -> Response {
    //     let height = ui.spacing().interact_size.y;
    //     let width = ui.spacing().interact_size.x;
    //     let mass_spectrum = self.data_frame[MASS_SPECTRUM].list().unwrap();
    //     let mass_spectrum_series = mass_spectrum.get_as_series(self.row_index).unwrap();
    //     ui.horizontal(|ui| {
    //         ui.label(format_list_truncated!(mass_spectrum_series.iter(), 2))
    //             .on_hover_ui(|ui| {
    //                 if let Ok(value) =
    //                     &self.data_frame[formatcp!("{MASS_SPECTRUM}.{COUNT}")].get(self.row_index)
    //                 {
    //                     ui.label(format!("Count: {value}"));
    //                 }
    //             })
    //             .on_hover_ui(|ui| {
    //                 ui.heading("Mass to charge");
    //                 if let Ok(value) =
    //                     &self.data_frame[formatcp!("{MASS_TO_CHARGE}.{MIN}")].get(self.row_index)
    //                 {
    //                     ui.label(format!("Min: {value}"));
    //                 }
    //                 if let Ok(value) =
    //                     &self.data_frame[formatcp!("{MASS_TO_CHARGE}.{MAX}")].get(self.row_index)
    //                 {
    //                     ui.label(format!("Max: {value}"));
    //                 }
    //             })
    //             .on_hover_ui(|ui| {
    //                 ui.heading("Signal");
    //                 if let Ok(value) =
    //                     &self.data_frame[formatcp!("{SIGNAL}.{MIN}")].get(self.row_index)
    //                 {
    //                     ui.label(format!("Min: {value}"));
    //                 }
    //                 if let Ok(value) =
    //                     &self.data_frame[formatcp!("{SIGNAL}.{MAX}")].get(self.row_index)
    //                 {
    //                     ui.label(format!("Max: {value}"));
    //                 }
    //                 if let Ok(value) =
    //                     &self.data_frame[formatcp!("{SIGNAL}.{SUM}")].get(self.row_index)
    //                 {
    //                     ui.label(format!("Sum: {value}"));
    //                 }
    //             })
    //             .on_hover_ui(|ui| {
    //                 ui.heading("???");
    //                 if let Ok(value) = &self.data_frame["_xy.cov"].get(self.row_index) {
    //                     ui.label(format!("Cov x,y: {value}"));
    //                 }
    //                 if let Ok(value) = &self.data_frame["Correlation"].get(self.row_index) {
    //                     ui.label(format!("Correlation: {value}"));
    //                 }
    //                 if let Ok(value) = &self.data_frame["Slope"].get(self.row_index) {
    //                     ui.label(format!("Slope: {value}"));
    //                 }
    //                 if let Ok(value) = &self.data_frame["Intercept"].get(self.row_index) {
    //                     ui.label(format!("Intercept: {value}"));
    //                 }
    //             });
    //         let mut space = ui.available_width();
    //         if ui.available_width() > height {
    //             space -= ui.spacing().button_padding.x + height;
    //         }
    //         ui.add_space(space);
    //         ui.visuals_mut().button_frame = false;
    //         ui.menu_button(RichText::new(LIST), |ui| {
    //             let total_rows = mass_spectrum_series.len();
    //             let mass_to_charge_signal = mass_spectrum_series.struct_().unwrap();
    //             let mass_to_charge_series =
    //                 mass_to_charge_signal.field_by_name(MASS_TO_CHARGE).unwrap();
    //             let signal_series = mass_to_charge_signal.field_by_name(SIGNAL).unwrap();
    //             TableBuilder::new(ui)
    //                 .cell_layout(Layout::centered_and_justified(Direction::LeftToRight))
    //                 .column(Column::auto_with_initial_suggestion(width))
    //                 .columns(Column::auto(), 2)
    //                 .auto_shrink([true, true])
    //                 .striped(true)
    //                 .header(height, |mut row| {
    //                     row.col(|ui| {
    //                         ui.heading("Index");
    //                     });
    //                     row.col(|ui| {
    //                         ui.heading("Mass to charge");
    //                     });
    //                     row.col(|ui| {
    //                         ui.heading("Signal");
    //                     });
    //                 })
    //                 .body(|body| {
    //                     body.rows(height, total_rows, |mut row| {
    //                         let row_index = row.index();
    //                         // Index
    //                         row.col(|ui| {
    //                             ui.label(row_index.to_string());
    //                         });
    //                         // Mass to charge
    //                         row.col(|ui| {
    //                             let mass_to_charge = mass_to_charge_series.f32().unwrap();
    //                             let value = mass_to_charge.get(row_index).unwrap();
    //                             let formated = self.settings.mass_to_charge.format(value);
    //                             ui.label(formated).on_hover_text(formated.precision(None));
    //                         });
    //                         // Signal
    //                         row.col(|ui| {
    //                             // let signal = signal_series.cast(&DataType::Float64).unwrap();
    //                             // let signal = signal.f64().unwrap();
    //                             // ui.label(signal.get(row_index).unwrap().to_string());
    //                             if self.settings.signal.normalize {
    //                                 let signal = signal_series.f64().unwrap();
    //                                 ui.add(
    //                                     SignalWidget::new(signal.get(row_index))
    //                                         .precision(Some(self.settings.signal.precision)),
    //                                 );
    //                             } else {
    //                                 let signal = signal_series.u16().unwrap();
    //                                 ui.add(
    //                                     SignalWidget::new(signal.get(row_index))
    //                                         .precision(Some(self.settings.signal.precision)),
    //                                 );
    //                             };
    //                         });
    //                     });
    //                 });
    //         });
    //     })
    //     .response
    // }
    pub fn show(self, ui: &mut Ui) -> PolarsResult<Response> {
        let height = ui.spacing().interact_size.y;
        let width = ui.spacing().interact_size.x;
        let meta = self.data_frame[META].struct_()?;
        let mass_spectrum = self.data_frame[MASS_SPECTRUM].list()?;
        let mass_spectrum_series = mass_spectrum
            .get_as_series(self.index)
            .ok_or(polars_err!(NoData: MASS_SPECTRUM))?;
        let response = ui
            .horizontal(|ui| -> PolarsResult<()> {
                ui.label(format_list_truncated!(mass_spectrum_series.iter(), 2))
                    .try_on_hover_ui(|ui| -> PolarsResult<()> {
                        ui.heading("Mass spectrum");
                        if let Ok(value) = &meta
                            .field_by_name(formatcp!("{MASS_SPECTRUM}.{LEN}"))?
                            .get(self.index)
                        {
                            ui.label(format!("Length: {value}"));
                        }
                        Ok(())
                    })?
                    .try_on_hover_ui(|ui| -> PolarsResult<()> {
                        ui.heading("Mass to charge");
                        if let Ok(value) = &meta
                            .field_by_name(formatcp!("{MASS_TO_CHARGE}.{MIN}"))?
                            .get(self.index)
                        {
                            ui.label(format!("Min: {value}"));
                        }
                        if let Ok(value) = &meta
                            .field_by_name(formatcp!("{MASS_TO_CHARGE}.{MAX}"))?
                            .get(self.index)
                        {
                            ui.label(format!("Max: {value}"));
                        }
                        Ok(())
                    })?
                    .try_on_hover_ui(|ui| -> PolarsResult<()> {
                        ui.heading("Signal");
                        if let Ok(value) = &meta
                            .field_by_name(formatcp!("{SIGNAL}.{MIN}"))?
                            .get(self.index)
                        {
                            ui.label(format!(
                                "Min: {value} (Normalized: {})",
                                meta.field_by_name(formatcp!("{SIGNAL}.{MIN}.{NORMALIZED}"))?
                                    .str_value(self.index)?
                            ));
                        }
                        if let Ok(value) = &meta
                            .field_by_name(formatcp!("{SIGNAL}.{MAX}"))?
                            .get(self.index)
                        {
                            ui.label(format!(
                                "Max: {value} (Normalized: {})",
                                meta.field_by_name(formatcp!("{SIGNAL}.{MAX}.{NORMALIZED}"))?
                                    .str_value(self.index)?
                            ));
                        }
                        if let Ok(value) = &meta
                            .field_by_name(formatcp!("{SIGNAL}.{SUM}"))?
                            .get(self.index)
                        {
                            ui.label(format!("Sum: {value}"));
                        }
                        Ok(())
                    })?
                    .try_on_hover_ui(|ui| -> PolarsResult<()> {
                        ui.heading("Rolling");
                        if let Ok(value) = &meta
                            .field_by_name(formatcp!("{ROLLING}.{COVARIANCE}"))?
                            .get(self.index)
                        {
                            ui.label(format!("Covariance: {value}"));
                        }
                        if let Ok(value) = &meta
                            .field_by_name(formatcp!("{ROLLING}.{CORRELATION}"))?
                            .get(self.index)
                        {
                            ui.label(format!("Correlation: {value}"));
                        }
                        if let Ok(value) = &meta
                            .field_by_name(formatcp!("{ROLLING}.{SLOPE}"))?
                            .get(self.index)
                        {
                            ui.label(format!("Slope: {value}"));
                        }
                        if let Ok(value) = &meta
                            .field_by_name(formatcp!("{ROLLING}.{INTERCEPT}"))?
                            .get(self.index)
                        {
                            ui.label(format!("Intercept: {value}"));
                        }
                        Ok(())
                    })?;
                let mut space = ui.available_width();
                if ui.available_width() > height {
                    space -= ui.spacing().button_padding.x + height;
                }
                ui.add_space(space);
                ui.visuals_mut().button_frame = false;
                ui.menu_button(RichText::new(LIST), |ui| {
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
                            let total_rows = mass_spectrum_series.len();
                            body.rows(height, total_rows, |row| {
                                _ = self.body(row);
                                // let row_index = row.index();
                                // // Index
                                // row.col(|ui| {
                                //     ui.label(row_index.to_string());
                                // });
                                // // Mass to charge
                                // row.col(|ui| {
                                //     let mass_to_charge = mass_to_charge_series.f32().unwrap();
                                //     let value = mass_to_charge.get(row_index).unwrap();
                                //     let formated = self.settings.mass_to_charge.format(value);
                                //     ui.label(formated).on_hover_text(formated.precision(None));
                                // });
                                // // Signal
                                // row.col(|ui| {
                                //     // let signal = signal_series.cast(&DataType::Float64).unwrap();
                                //     // let signal = signal.f64().unwrap();
                                //     // ui.label(signal.get(row_index).unwrap().to_string());
                                //     if self.settings.signal.normalize {
                                //         let signal = signal_series.f64().unwrap();
                                //         ui.add(
                                //             SignalWidget::new(signal.get(row_index))
                                //                 .precision(Some(self.settings.signal.precision)),
                                //         );
                                //     } else {
                                //         let signal = signal_series.u16().unwrap();
                                //         ui.add(
                                //             SignalWidget::new(signal.get(row_index))
                                //                 .precision(Some(self.settings.signal.precision)),
                                //         );
                                //     };
                                // });
                            });
                        });
                });
                Ok(())
            })
            .response;
        Ok(response)
    }

    #[instrument(skip_all, err)]
    fn body(&self, mut row: TableRow<'_, '_>) -> PolarsResult<()> {
        let row_index = row.index();
        let mass_spectrum = self.data_frame[MASS_SPECTRUM].list()?;
        let mass_spectrum_series = mass_spectrum
            .get_as_series(self.index)
            .ok_or(polars_err!(NoData: MASS_SPECTRUM))?;
        // Index
        row.col(|ui| {
            ui.label(row_index.to_string());
        });
        // Mass to charge
        let mass_to_charge_series = mass_spectrum_series
            .struct_()?
            .field_by_name(MASS_TO_CHARGE)?;
        let mass_to_charge = mass_to_charge_series.f64()?;
        row.col(|ui| {
            let text =
                mass_to_charge
                    .get(row_index)
                    .map_or(WidgetText::from(EM_DASH), |mass_to_charge| {
                        WidgetText::from(format!("{mass_to_charge:.0$}", self.settings.precision))
                    });
            ui.label(text);
        });
        // Signal
        let signal_series = mass_spectrum_series.struct_()?.field_by_name(SIGNAL)?;
        let signal = signal_series.f64()?;
        row.col(|ui| {
            let text = signal
                .get(row_index)
                .map_or(WidgetText::from(EM_DASH), |signal| {
                    WidgetText::from(signal.to_string())
                });
            ui.label(text);
        });
        Ok(())
    }
}

impl Widget for MassSpectrum<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui).expect("show mass spectrum widget")
    }
}
