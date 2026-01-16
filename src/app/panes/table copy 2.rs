use crate::{
    app::{states::pane::State, widgets::mass_spectrum::MassSpectrum},
    r#const::*,
    utils::hash::{HashedDataFrame, HashedMetaDataFrame},
};
use const_format::formatcp;
use egui::{
    CentralPanel, Direction, Frame, Id, Layout, MenuBar, RichText, ScrollArea, TextStyle,
    TopBottomPanel, Ui,
};
use egui_ext::ResponseExt;
use egui_extras::{Column, TableBuilder};
use egui_phosphor::regular::{COPY, COPY_SIMPLE, TAG, X};
use egui_tiles::{TileId, UiResponse};
use metadata::egui::MetadataWidget;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::error;

const COLUMN_COUNT: usize = 3;

/// Table view
pub(crate) struct TableView<'a> {
    data: &'a HashedDataFrame,
    state: &'a mut State,
}

impl<'a> TableView<'a> {
    pub(crate) fn new(data: &'a HashedDataFrame, state: &'a mut State) -> Self {
        Self { data, state }
    }
}

impl TableView<'_> {
    pub(super) fn show(&mut self, ui: &mut Ui) {
        _ = self.grouped_by_retention_time(ui);
        // if let Err(error) = match self.state.settings.sort {
        //     Sort::RetentionTime if !self.state.settings.explode => {
        //         self.grouped_by_retention_time(ui)
        //     }
        //     Sort::MassToCharge if !self.state.settings.explode => {
        //         self.grouped_by_mass_to_charge(ui)
        //     }
        //     _ => self.exploded(ui),
        // } {
        //     error!(%error);
        //     ui.label(error.to_string());
        // }
    }

    // fn grouped_by_mass_to_charge(&self, ui: &mut Ui) -> PolarsResult<()> {
    //     let width = ui.spacing().interact_size.x;
    //     let height = ui.spacing().interact_size.y;
    //     let data_frame = ui.memory_mut(|memory| {
    //         memory
    //             .caches
    //             .cache::<TableComputed>()
    //             .get(TableKey::new(&self.frame.data, &self.settings))
    //     });
    //     let total_rows = data_frame.height();
    //     // let mass_to_charge = .cast(&DataType::UInt32)?;
    //     let mass_to_charge = data_frame[MASS_TO_CHARGE]
    //         .as_materialized_series()
    //         .round(2, RoundMode::HalfToEven)?;
    //     let mass_to_charge = mass_to_charge.f32()?;
    //     TableBuilder::new(ui)
    //         .cell_layout(Layout::centered_and_justified(Direction::LeftToRight))
    //         .column(Column::auto_with_initial_suggestion(width))
    //         .columns(Column::auto(), COLUMN_COUNT - 1)
    //         .auto_shrink(false)
    //         .striped(true)
    //         .header(height, |mut row| {
    //             row.col(|ui| {
    //                 ui.heading("Index");
    //             });
    //             row.col(|ui| {
    //                 ui.heading("Mass to charge");
    //             });
    //             row.col(|ui| {
    //                 ui.heading("Extracted ion chromatogram");
    //             });
    //         })
    //         .body(|body| {
    //             body.rows(height, total_rows, |mut row| {
    //                 let row_index = row.index();
    //                 // Index
    //                 row.col(|ui| {
    //                     ui.label(row_index.to_string());
    //                 });
    //                 // Mass to charge
    //                 row.col(|ui| {
    //                     if let Some(value) = mass_to_charge.get(row_index) {
    //                         // let formated = self.settings.mass_to_charge.format(value);
    //                         ui.label(value.to_string()).on_hover_text(value.to_string());
    //                     } else {
    //                         ui.label(AnyValue::Null.to_string());
    //                     }
    //                 });
    //                 // EIC
    //                 row.col(|ui| {
    //                     ui.add(IonChromatogram {
    //                         data_frame: &data_frame,
    //                         row_index,
    //                         settings: &self.settings,
    //                     });
    //                 });
    //             });
    //         });
    //     Ok(())
    // }

    fn grouped_by_retention_time(&self, ui: &mut Ui) -> PolarsResult<()> {
        let width = ui.spacing().interact_size.x;
        let height = ui.spacing().interact_size.y;
        let total_rows = self.data.height();
        let retention_time = self.data[RETENTION_TIME].as_materialized_series();
        TableBuilder::new(ui)
            .cell_layout(Layout::centered_and_justified(Direction::LeftToRight))
            .column(Column::auto_with_initial_suggestion(width))
            .columns(Column::auto(), COLUMN_COUNT - 1)
            .auto_shrink(false)
            .striped(true)
            .header(height, |mut row| {
                row.col(|ui| {
                    ui.heading("Index");
                });
                row.col(|ui| {
                    ui.heading("RetentionTime");
                });
                row.col(|ui| {
                    ui.heading("MassSpectrum");
                });
            })
            .body(|body| {
                body.rows(height, total_rows, |mut row| {
                    let row_index = row.index();
                    // Index
                    row.col(|ui| {
                        _ = threshold(&self.data, row_index, ui);
                        ui.label(row_index.to_string());
                    });
                    // Retention time
                    row.col(|ui| {
                        _ = threshold(&self.data, row_index, ui);
                        let meta = &self.data[META];
                        let physical_retention_time =
                            physical_retention_time(retention_time, row_index).unwrap();
                        ui.label(retention_time.str_value(row_index).unwrap())
                            .on_hover_text(physical_retention_time.to_string())
                            .try_on_hover_ui(|ui| -> PolarsResult<()> {
                                ui.heading("Ions");
                                let base_peak = meta
                                    .struct_()?
                                    .field_by_name(formatcp!("{MASS_SPECTRUM}.BasePeak"))?;
                                ui.label(format!("Base: {}", base_peak.str_value(row_index)?));
                                let molecular_ion = meta
                                    .struct_()?
                                    .field_by_name(formatcp!("{MASS_SPECTRUM}.MolecularPeak"))?;
                                ui.label(format!(
                                    "Molecular: {}",
                                    molecular_ion.str_value(row_index)?
                                ));
                                let ion55 =
                                    meta.struct_()?.field_by_name(formatcp!("{SIGNAL}.Ion55"))?;
                                ui.label(format!("55: {}", ion55.str_value(row_index)?));
                                let ion67 =
                                    meta.struct_()?.field_by_name(formatcp!("{SIGNAL}.Ion67"))?;
                                ui.label(format!("67: {}", ion67.str_value(row_index)?));
                                let ion74 =
                                    meta.struct_()?.field_by_name(formatcp!("{SIGNAL}.Ion74"))?;
                                ui.label(format!("74: {}", ion74.str_value(row_index)?));
                                let ion79 =
                                    meta.struct_()?.field_by_name(formatcp!("{SIGNAL}.Ion79"))?;
                                ui.label(format!("79: {}", ion79.str_value(row_index)?));
                                let ion81 =
                                    meta.struct_()?.field_by_name(formatcp!("{SIGNAL}.Ion81"))?;
                                ui.label(format!("81: {}", ion81.str_value(row_index)?));
                                let ion87 =
                                    meta.struct_()?.field_by_name(formatcp!("{SIGNAL}.Ion87"))?;
                                ui.label(format!("87: {}", ion87.str_value(row_index)?));
                                let ion91 =
                                    meta.struct_()?.field_by_name(formatcp!("{SIGNAL}.Ion91"))?;
                                ui.label(format!("91: {}", ion91.str_value(row_index)?));
                                let ion108 = meta
                                    .struct_()?
                                    .field_by_name(formatcp!("{SIGNAL}.Ion108"))?;
                                ui.label(format!("108: {}", ion108.str_value(row_index)?));
                                let ion150 = meta
                                    .struct_()?
                                    .field_by_name(formatcp!("{SIGNAL}.Ion150"))?;
                                ui.label(format!("150: {}", ion150.str_value(row_index)?));
                                Ok(())
                            })
                            .unwrap()
                            .try_on_hover_ui(|ui| -> PolarsResult<()> {
                                let is_saturated = meta
                                    .struct_()?
                                    .field_by_name(formatcp!("{MASS_SPECTRUM}.IsSaturated"))?;
                                ui.label(format!(
                                    "IsSaturated: {}",
                                    is_saturated.str_value(row_index)?
                                ));
                                let is_monoenoic = meta
                                    .struct_()?
                                    .field_by_name(formatcp!("{MASS_SPECTRUM}.IsMonoenoic"))?;
                                ui.label(format!(
                                    "IsMonoenoic: {}",
                                    is_monoenoic.str_value(row_index)?
                                ));
                                let is_dienoic = meta
                                    .struct_()?
                                    .field_by_name(formatcp!("{MASS_SPECTRUM}.IsDienoic"))?;
                                ui.label(format!(
                                    "IsDienoic: {}",
                                    is_dienoic.str_value(row_index)?
                                ));
                                let is_polyenoic = meta
                                    .struct_()?
                                    .field_by_name(formatcp!("{MASS_SPECTRUM}.IsPolyenoic"))?;
                                ui.label(format!(
                                    "IsPolyenoic: {}",
                                    is_polyenoic.str_value(row_index)?
                                ));
                                let is_tropylium = meta
                                    .struct_()?
                                    .field_by_name(formatcp!("{MASS_SPECTRUM}.IsTropylium"))?;
                                ui.label(format!(
                                    "IsTropylium: {}",
                                    is_tropylium.str_value(row_index)?
                                ));
                                let is_omega_3 = meta
                                    .struct_()?
                                    .field_by_name(formatcp!("{MASS_SPECTRUM}.IsOmega-3"))?;
                                ui.label(format!(
                                    "IsOmega-3: {}",
                                    is_omega_3.str_value(row_index)?
                                ));
                                let is_omega_6 = meta
                                    .struct_()?
                                    .field_by_name(formatcp!("{MASS_SPECTRUM}.IsOmega-6"))?;
                                ui.label(format!(
                                    "IsOmega-6: {}",
                                    is_omega_6.str_value(row_index)?
                                ));
                                Ok(())
                            })
                            .unwrap()
                            .context_menu(|ui| {
                                if ui.button((COPY, "Copy")).clicked() {
                                    ui.ctx().copy_text(physical_retention_time.to_string());
                                }
                            });
                    });
                    // Mass spectrum
                    row.col(|ui| {
                        _ = threshold(&self.data, row_index, ui);
                        _ = MassSpectrum {
                            data_frame: &self.data,
                            index: row_index,
                            settings: &self.state.settings,
                        }
                        .show(ui);
                    });
                });
            });
        Ok(())
    }

    // fn exploded(&self, ui: &mut Ui) -> PolarsResult<()> {
    //     let width = ui.spacing().interact_size.x;
    //     let height = ui.spacing().interact_size.y;
    //     let data_frame = ui.memory_mut(|memory| {
    //         memory
    //             .caches
    //             .cache::<TableComputed>()
    //             .get(TableKey::new(&self.frame.data, &self.settings))
    //     });
    //     let total_rows = data_frame.height();
    //     let retention_time = data_frame[RETENTION_TIME].i32()?;
    //     let mass_to_charge = data_frame[MASS_TO_CHARGE].f32()?;
    //     let signal = data_frame[SIGNAL].u16()?;
    //     TableBuilder::new(ui)
    //         .cell_layout(Layout::centered_and_justified(Direction::LeftToRight))
    //         .column(Column::auto_with_initial_suggestion(width))
    //         .columns(Column::auto(), COLUMN_COUNT)
    //         .auto_shrink(false)
    //         .striped(true)
    //         .header(height, |mut row| {
    //             row.col(|ui| {
    //                 ui.heading("Index");
    //             });
    //             let retention_time = |ui: &mut Ui| {
    //                 ui.heading("Retention time");
    //             };
    //             let mass_to_charge = |ui: &mut Ui| {
    //                 ui.heading("Mass to charge");
    //             };
    //             match self.settings.sort {
    //                 Sort::RetentionTime => {
    //                     row.col(retention_time);
    //                     row.col(mass_to_charge);
    //                 }
    //                 Sort::MassToCharge => {
    //                     row.col(mass_to_charge);
    //                     row.col(retention_time);
    //                 }
    //             }
    //             row.col(|ui| {
    //                 ui.heading(SIGNAL);
    //             });
    //         })
    //         .body(|body| {
    //             body.rows(height, total_rows, |mut row| {
    //                 let row_index = row.index();
    //                 // Index
    //                 row.col(|ui| {
    //                     ui.label(row_index.to_string());
    //                 });
    //                 // RetentionTime & MassToCharge
    //                 let retention_time = |ui: &mut Ui| {
    //                     if let Some(value) = retention_time.get(row_index) {
    //                         let formated = self.settings.retention_time.format(value as _);
    //                         ui.label(formated).on_hover_text(formated.precision(None));
    //                         // let time = Time::new::<millisecond>(value as _);
    //                         // let value = match self.settings.retention_time.units {
    //                         //     TimeUnits::Millisecond => time.get::<millisecond>(),
    //                         //     TimeUnits::Second => time.get::<second>(),
    //                         //     TimeUnits::Minute => time.get::<minute>(),
    //                         // };
    //                         // ui.label(format!(
    //                         //     "{value:.*}",
    //                         //     self.settings.retention_time.precision,
    //                         // ))
    //                         // .on_hover_text(format!("{value}"));
    //                     }
    //                 };
    //                 let mass_to_charge = |ui: &mut Ui| {
    //                     if let Some(value) = mass_to_charge.get(row_index) {
    //                         ui.label(format!(
    //                             "{value:.*}",
    //                             self.settings.mass_to_charge.precision,
    //                         ))
    //                         .on_hover_text(format!("{value}"));
    //                     }
    //                 };
    //                 match self.settings.sort {
    //                     Sort::RetentionTime => {
    //                         row.col(retention_time);
    //                         row.col(mass_to_charge);
    //                     }
    //                     Sort::MassToCharge => {
    //                         row.col(mass_to_charge);
    //                         row.col(retention_time);
    //                     }
    //                 }
    //                 // Signal
    //                 row.col(|ui| {
    //                     if let Some(value) = signal.get(row_index) {
    //                         ui.label(format!("{value}"))
    //                             .on_hover_text(format!("{value}"));
    //                     }
    //                 });
    //             });
    //         });
    //     Ok(())
    // }
}

fn physical_retention_time(retention_time: &Series, row: usize) -> PolarsResult<i64> {
    retention_time
        .duration()?
        .physical()
        .get(row)
        .ok_or(polars_err!(NoData: "RetentionTime"))
}

pub fn threshold(data_frame: &DataFrame, row: usize, ui: &mut Ui) -> PolarsResult<()> {
    if let Some(threshold) = data_frame[META]
        .struct_()?
        .field_by_name(THRESHOLD)?
        .bool()?
        .get(row)
        && !threshold
    {
        ui.multiply_opacity(ui.visuals().disabled_alpha());
    }
    Ok(())
}
