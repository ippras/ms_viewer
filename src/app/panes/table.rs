use crate::{
    app::{states::pane::State, widgets::mass_spectrum::MassSpectrum},
    r#const::*,
    utils::hash::{HashedDataFrame, HashedMetaDataFrame},
};
use const_format::formatcp;
use egui::{
    CentralPanel, Direction, Frame, Id, Layout, Margin, MenuBar, RichText, ScrollArea, TextStyle,
    TextWrapMode, TopBottomPanel, Ui,
};
use egui_ext::ResponseExt;
use egui_l20n::prelude::*;
use egui_phosphor::regular::{COPY, COPY_SIMPLE, HASH, TAG, X};
use egui_table::{CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate, TableState};
use egui_tiles::{TileId, UiResponse};
use metadata::egui::MetadataWidget;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::Range;
use tracing::{error, instrument};

pub(crate) const ID_SOURCE: &str = "Table";

const COLUMN_COUNT: usize = 3;
const LEN: usize = top::MASS_SPECTRUM.end;
const TOP: &[Range<usize>] = &[top::INDEX, top::RETENTION_TIME, top::MASS_SPECTRUM];

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

// let total_rows = self.data.height();
impl TableView<'_> {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        let id_salt = Id::new(ID_SOURCE).with("Table");
        if self.state.event.reset_table_state {
            let id = TableState::id(ui, Id::new(id_salt));
            TableState::reset(ui.ctx(), id);
            self.state.event.reset_table_state = false;
        }
        let height = ui.text_style_height(&TextStyle::Heading) + 2.0 * MARGIN.y;
        let num_rows = self.data.height() as u64;
        let num_columns = LEN;
        Table::new()
            .id_salt(id_salt)
            .num_rows(num_rows)
            .columns(vec![
                Column::default()
                    .resizable(self.state.settings.table.resizable);
                num_columns
            ])
            .num_sticky_cols(self.state.settings.table.sticky_columns)
            .headers([
                HeaderRow {
                    height,
                    groups: TOP.to_vec(),
                },
                HeaderRow::new(height),
            ])
            .show(ui, self);
    }

    fn header_cell_content_ui(&mut self, ui: &mut Ui, row: usize, column: Range<usize>) {
        if self.state.settings.table.truncate_headers {
            ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
        }
        match (row, column) {
            // Top
            (0, top::INDEX) => {
                ui.heading(HASH).on_hover_localized("Index");
            }
            (0, top::RETENTION_TIME) => {
                ui.heading(ui.localize("RetentionTime"));
            }
            (0, top::MASS_SPECTRUM) => {
                ui.heading(ui.localize("MassSpectrum"));
            }
            _ => {}
        };
    }

    #[instrument(skip(self, ui), err)]
    fn cell_content_ui(
        &mut self,
        ui: &mut Ui,
        row: usize,
        column: Range<usize>,
    ) -> PolarsResult<()> {
        threshold(&self.data, row, ui)?;
        match (row, column) {
            (row, top::INDEX) => {
                ui.label(row.to_string());
            }
            (row, top::RETENTION_TIME) => self.retention_time(ui, row)?,
            (row, top::MASS_SPECTRUM) => self.mass_spectrum(ui, row)?,
        }
        Ok(())
    }
}

impl TableDelegate for TableView<'_> {
    fn header_cell_ui(&mut self, ui: &mut Ui, cell: &HeaderCellInfo) {
        Frame::new()
            .inner_margin(Margin::from(MARGIN))
            .show(ui, |ui| {
                self.header_cell_content_ui(ui, cell.row_nr, cell.col_range.clone())
            });
    }

    fn cell_ui(&mut self, ui: &mut Ui, cell: &CellInfo) {
        if cell.row_nr.is_multiple_of(2) {
            ui.painter()
                .rect_filled(ui.max_rect(), 0.0, ui.visuals().faint_bg_color);
        }
        Frame::new()
            .inner_margin(Margin::from(MARGIN))
            .show(ui, |ui| {
                _ = self.cell_content_ui(ui, cell.row_nr as _, cell.col_nr..cell.col_nr + 1);
            });
    }
}

impl TableView<'_> {
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

    fn mass_spectrum(&self, ui: &mut Ui, row: usize) -> PolarsResult<()> {
        MassSpectrum {
            data_frame: &self.data,
            index: row,
            settings: &self.state.settings,
        }
        .show(ui)?;
        Ok(())
    }

    fn retention_time(&self, ui: &mut Ui, row: usize) -> PolarsResult<()> {
        let meta = &self.data[META];
        let text = retention_time
            .str_value(row)
            .ok_or(polars_err!(NoData: RETENTION_TIME));
        // let physical_retention_time = physical_retention_time(retention_time, row).unwrap();
        ui.label(text)
            .on_hover_text(physical_retention_time.to_string())
            .try_on_hover_ui(|ui| -> PolarsResult<()> {
                ui.heading("Ions");
                let base_peak = meta
                    .struct_()?
                    .field_by_name(formatcp!("{MASS_SPECTRUM}.BasePeak"))?;
                ui.label(format!("Base: {}", base_peak.str_value(row)?));
                let molecular_ion = meta
                    .struct_()?
                    .field_by_name(formatcp!("{MASS_SPECTRUM}.MolecularPeak"))?;
                ui.label(format!("Molecular: {}", molecular_ion.str_value(row)?));
                let ion55 = meta.struct_()?.field_by_name(formatcp!("{SIGNAL}.Ion55"))?;
                ui.label(format!("55: {}", ion55.str_value(row)?));
                let ion67 = meta.struct_()?.field_by_name(formatcp!("{SIGNAL}.Ion67"))?;
                ui.label(format!("67: {}", ion67.str_value(row)?));
                let ion74 = meta.struct_()?.field_by_name(formatcp!("{SIGNAL}.Ion74"))?;
                ui.label(format!("74: {}", ion74.str_value(row)?));
                let ion79 = meta.struct_()?.field_by_name(formatcp!("{SIGNAL}.Ion79"))?;
                ui.label(format!("79: {}", ion79.str_value(row)?));
                let ion81 = meta.struct_()?.field_by_name(formatcp!("{SIGNAL}.Ion81"))?;
                ui.label(format!("81: {}", ion81.str_value(row)?));
                let ion87 = meta.struct_()?.field_by_name(formatcp!("{SIGNAL}.Ion87"))?;
                ui.label(format!("87: {}", ion87.str_value(row)?));
                let ion91 = meta.struct_()?.field_by_name(formatcp!("{SIGNAL}.Ion91"))?;
                ui.label(format!("91: {}", ion91.str_value(row)?));
                let ion108 = meta
                    .struct_()?
                    .field_by_name(formatcp!("{SIGNAL}.Ion108"))?;
                ui.label(format!("108: {}", ion108.str_value(row)?));
                let ion150 = meta
                    .struct_()?
                    .field_by_name(formatcp!("{SIGNAL}.Ion150"))?;
                ui.label(format!("150: {}", ion150.str_value(row)?));
                Ok(())
            })?
            .try_on_hover_ui(|ui| -> PolarsResult<()> {
                let is_saturated = meta
                    .struct_()?
                    .field_by_name(formatcp!("{MASS_SPECTRUM}.IsSaturated"))?;
                ui.label(format!("IsSaturated: {}", is_saturated.str_value(row)?));
                let is_monoenoic = meta
                    .struct_()?
                    .field_by_name(formatcp!("{MASS_SPECTRUM}.IsMonoenoic"))?;
                ui.label(format!("IsMonoenoic: {}", is_monoenoic.str_value(row)?));
                let is_dienoic = meta
                    .struct_()?
                    .field_by_name(formatcp!("{MASS_SPECTRUM}.IsDienoic"))?;
                ui.label(format!("IsDienoic: {}", is_dienoic.str_value(row)?));
                let is_polyenoic = meta
                    .struct_()?
                    .field_by_name(formatcp!("{MASS_SPECTRUM}.IsPolyenoic"))?;
                ui.label(format!("IsPolyenoic: {}", is_polyenoic.str_value(row)?));
                let is_tropylium = meta
                    .struct_()?
                    .field_by_name(formatcp!("{MASS_SPECTRUM}.IsTropylium"))?;
                ui.label(format!("IsTropylium: {}", is_tropylium.str_value(row)?));
                let is_omega_3 = meta
                    .struct_()?
                    .field_by_name(formatcp!("{MASS_SPECTRUM}.IsOmega-3"))?;
                ui.label(format!("IsOmega-3: {}", is_omega_3.str_value(row)?));
                let is_omega_6 = meta
                    .struct_()?
                    .field_by_name(formatcp!("{MASS_SPECTRUM}.IsOmega-6"))?;
                ui.label(format!("IsOmega-6: {}", is_omega_6.str_value(row)?));
                Ok(())
            })?
            .context_menu(|ui| {
                if ui.button((COPY, "Copy")).clicked() {
                    ui.ctx().copy_text(physical_retention_time.to_string());
                }
            });
        Ok(())
    }

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

mod top {
    use super::*;

    pub(super) const INDEX: Range<usize> = 0..1;
    pub(super) const RETENTION_TIME: Range<usize> = INDEX.end..INDEX.end + 1;
    pub(super) const MASS_SPECTRUM: Range<usize> = RETENTION_TIME.end..RETENTION_TIME.end + 1;
}
