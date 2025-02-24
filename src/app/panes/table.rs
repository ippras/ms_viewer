use super::{
    settings::{Settings, Sort, TimeUnits},
    widgets::{ion_chromatogram::IonChromatogram, mass_spectrum::MassSpectrum},
};
use crate::app::computers::{TableComputed, TableKey};
use egui::{Direction, Layout, Ui};
use egui_extras::{Column, TableBuilder};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::error;
use uom::si::{
    f32::Time,
    time::{millisecond, minute, second},
};

const COLUMN_COUNT: usize = 3;

/// Table pane
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct TablePane {
    pub(crate) data_frame: DataFrame,
    pub(crate) settings: Settings,
}

impl TablePane {
    pub(super) fn ui(&self, ui: &mut Ui) {
        if let Err(error) = match self.settings.sort {
            Sort::RetentionTime if !self.settings.explode => self.grouped_by_retention_time(ui),
            Sort::MassToCharge if !self.settings.explode => self.grouped_by_mass_to_charge(ui),
            _ => self.exploded(ui),
        } {
            error!(%error);
            ui.label(error.to_string());
        }
    }

    fn grouped_by_mass_to_charge(&self, ui: &mut Ui) -> PolarsResult<()> {
        let width = ui.spacing().interact_size.x;
        let height = ui.spacing().interact_size.y;
        let data_frame = ui.memory_mut(|memory| {
            memory.caches.cache::<TableComputed>().get(TableKey {
                data_frame: &self.data_frame,
                settings: &self.settings,
            })
        });
        let total_rows = data_frame.height();
        // let mass_to_charge = .cast(&DataType::UInt32)?;
        let mass_to_charge = data_frame["MassToCharge"]
            .as_materialized_series()
            .round(2)?;
        let mass_to_charge = mass_to_charge.f32()?;
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
                    ui.heading("Mass to charge");
                });
                row.col(|ui| {
                    ui.heading("Extracted ion chromatogram");
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
                        if let Some(value) = mass_to_charge.get(row_index) {
                            let formated = self.settings.mass_to_charge.format(value);
                            ui.label(formated).on_hover_text(formated.precision(None));
                        } else {
                            ui.label(AnyValue::Null.to_string());
                        }
                    });
                    // EIC
                    row.col(|ui| {
                        ui.add(IonChromatogram {
                            data_frame: &data_frame,
                            row_index,
                            settings: &self.settings,
                        });
                    });
                });
            });
        Ok(())
    }

    fn grouped_by_retention_time(&self, ui: &mut Ui) -> PolarsResult<()> {
        let width = ui.spacing().interact_size.x;
        let height = ui.spacing().interact_size.y;
        let data_frame = ui.memory_mut(|memory| {
            memory.caches.cache::<TableComputed>().get(TableKey {
                data_frame: &self.data_frame,
                settings: &self.settings,
            })
        });
        let total_rows = data_frame.height();
        let retention_time = data_frame["RetentionTime"].i32()?;
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
                    ui.heading("Retention time");
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
                        ui.label(row_index.to_string());
                    });
                    // Retention time
                    row.col(|ui| {
                        if let Some(value) = retention_time.get(row_index) {
                            let formated = self.settings.retention_time.format(value as _);
                            ui.label(formated).on_hover_text(formated.precision(None));
                        }
                    });
                    // Mass spectrum
                    row.col(|ui| {
                        ui.add(MassSpectrum {
                            data_frame: &data_frame,
                            row_index,
                            settings: &self.settings,
                        });
                    });
                });
            });
        Ok(())
    }

    fn exploded(&self, ui: &mut Ui) -> PolarsResult<()> {
        let width = ui.spacing().interact_size.x;
        let height = ui.spacing().interact_size.y;
        let data_frame = ui.memory_mut(|memory| {
            memory.caches.cache::<TableComputed>().get(TableKey {
                data_frame: &self.data_frame,
                settings: &self.settings,
            })
        });
        let total_rows = data_frame.height();
        let retention_time = data_frame["RetentionTime"].i32()?;
        let mass_to_charge = data_frame["MassToCharge"].f32()?;
        let signal = data_frame["Signal"].u16()?;
        TableBuilder::new(ui)
            .cell_layout(Layout::centered_and_justified(Direction::LeftToRight))
            .column(Column::auto_with_initial_suggestion(width))
            .columns(Column::auto(), COLUMN_COUNT)
            .auto_shrink(false)
            .striped(true)
            .header(height, |mut row| {
                row.col(|ui| {
                    ui.heading("Index");
                });
                let retention_time = |ui: &mut Ui| {
                    ui.heading("Retention time");
                };
                let mass_to_charge = |ui: &mut Ui| {
                    ui.heading("Mass to charge");
                };
                match self.settings.sort {
                    Sort::RetentionTime => {
                        row.col(retention_time);
                        row.col(mass_to_charge);
                    }
                    Sort::MassToCharge => {
                        row.col(mass_to_charge);
                        row.col(retention_time);
                    }
                }
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
                    // RetentionTime & MassToCharge
                    let retention_time = |ui: &mut Ui| {
                        if let Some(value) = retention_time.get(row_index) {
                            let formated = self.settings.retention_time.format(value as _);
                            ui.label(formated).on_hover_text(formated.precision(None));
                            // let time = Time::new::<millisecond>(value as _);
                            // let value = match self.settings.retention_time.units {
                            //     TimeUnits::Millisecond => time.get::<millisecond>(),
                            //     TimeUnits::Second => time.get::<second>(),
                            //     TimeUnits::Minute => time.get::<minute>(),
                            // };
                            // ui.label(format!(
                            //     "{value:.*}",
                            //     self.settings.retention_time.precision,
                            // ))
                            // .on_hover_text(format!("{value}"));
                        }
                    };
                    let mass_to_charge = |ui: &mut Ui| {
                        if let Some(value) = mass_to_charge.get(row_index) {
                            ui.label(format!(
                                "{value:.*}",
                                self.settings.mass_to_charge.precision,
                            ))
                            .on_hover_text(format!("{value}"));
                        }
                    };
                    match self.settings.sort {
                        Sort::RetentionTime => {
                            row.col(retention_time);
                            row.col(mass_to_charge);
                        }
                        Sort::MassToCharge => {
                            row.col(mass_to_charge);
                            row.col(retention_time);
                        }
                    }
                    // Signal
                    row.col(|ui| {
                        if let Some(value) = signal.get(row_index) {
                            ui.label(format!("{value}"))
                                .on_hover_text(format!("{value}"));
                        }
                    });
                });
            });
        Ok(())
    }
}

pub fn retention_time(units: TimeUnits) -> impl Fn(Option<f32>) -> Option<f32> + Copy {
    move |value| {
        let time = Time::new::<millisecond>(value?);
        Some(match units {
            TimeUnits::Millisecond => time.get::<millisecond>(),
            TimeUnits::Second => time.get::<second>(),
            TimeUnits::Minute => time.get::<minute>(),
        })
    }
}
