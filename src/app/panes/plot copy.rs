use crate::{
    app::{
        computers::{
            plot::{Computed as PlotComputed, Key as PlotKey},
            table::{Computed as TableComputed, Key as TableKey},
        },
        states::settings::{Settings, Sort},
    },
    r#const::*,
    utils::hash::{HashedDataFrame, HashedMetaDataFrame},
};
use egui::{
    Align2, RichText, Ui, Vec2,
    emath::{Float, round_to_decimals},
};
use egui_ext::color;
use egui_plot::{Bar, BarChart, Legend, Line, Plot, PlotMemory, PlotPoint, PlotPoints, Text};
use indexmap::IndexMap;
use polars::{error::PolarsResult, frame::DataFrame};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    iter::{empty, zip},
};
use tracing::error;

/// Plot pane
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct PlotPane {
    pub(crate) frame: HashedMetaDataFrame,
    // pub(crate) computed: HashedDataFrame,
    pub(crate) settings: Settings,
}

impl PlotPane {
    pub(super) fn ui(&mut self, ui: &mut Ui) {
        match self.settings.sort {
            Sort::RetentionTime if !self.settings.explode => self.grouped_by_retention_time(ui),
            Sort::MassToCharge if !self.settings.explode => self.grouped_by_mass_to_charge(ui),
            _ => unimplemented!(),
        }
    }

    pub(super) fn grouped_by_mass_to_charge(&self, ui: &mut Ui) {
        let data_frame = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<TableComputed>()
                .get(TableKey::new(&self.frame.data, &self.settings))
        });
        // let points = data_frame.height();
        let mass_to_charge = data_frame[MASS_TO_CHARGE].f32().unwrap();
        let retention_time = data_frame[RETENTION_TIME].list().unwrap();
        let signal = data_frame[SIGNAL].list().unwrap();
        ui.vertical_centered_justified(|ui| {
            // let id = ui.make_persistent_id("plot");
            // let plot_memory = PlotMemory::load(ui.ctx(), id);
            let mut plot = Plot::new("plot")
                .y_axis_formatter(move |y, _| round_to_decimals(y.value, 5).to_string());
            if self.settings.legend {
                let mut legend = Legend::default();
                // if let Some(visible) = self.settings.visible.take() {
                //     legend = if visible {
                //         legend.hidden_items(empty())
                //     } else {
                //         let hidden_items = mass_to_charge
                //             .iter()
                //             .filter_map(|mass_to_charge| Some(mass_to_charge?.to_string()));
                //         legend.hidden_items(hidden_items)
                //     };
                // }
                plot = plot.legend(legend);
            }
            plot.show(ui, |ui| {
                // let bounds = ui.plot_bounds().range_x();
                // let width = ui.plot_bounds().width();
                // tracing::error!(?width);

                // Lines
                for (mass_to_charge, retention_time, signal) in
                    zip(mass_to_charge, zip(retention_time, signal)).filter_map(
                        |(mass_to_charge, (retention_time, signal))| {
                            Some((mass_to_charge?, retention_time?, signal?))
                        },
                    )
                {
                    let line = Line::new(
                        mass_to_charge.to_string(),
                        PlotPoints::from_iter(
                            zip(retention_time.i32().unwrap(), signal.u16().unwrap()).filter_map(
                                |(retention_time, signal)| {
                                    Some([retention_time? as _, signal? as _])
                                },
                            ),
                        ),
                    );
                    ui.line(line);
                }

                // // Bars
                // for (mass_to_charge, retention_time, signal) in
                //     zip(mass_to_charge, zip(retention_time, signal)).filter_map(
                //         |(mass_to_charge, (retention_time, signal))| {
                //             Some((mass_to_charge?, retention_time?, signal?))
                //         },
                //     )
                // {
                //     let bars = zip(retention_time.i32().unwrap(), signal.u16().unwrap())
                //         .filter_map(|(retention_time, signal)| {
                //             Some(Bar::new(retention_time? as _, signal? as _))
                //         })
                //         .collect();
                //     let chart = BarChart::new(bars).name(mass_to_charge.to_string());
                //     ui.bar_chart(chart);
                // }

                // let mut charts = Vec::new();
                // for (mass_to_charge, retention_time, signal) in
                //     zip(mass_to_charge, zip(retention_time, signal)).filter_map(
                //         |(mass_to_charge, (retention_time, signal))| {
                //             Some((mass_to_charge?, retention_time?, signal?))
                //         },
                //     )
                // {
                //     let bars = zip(retention_time.i32().unwrap(), signal.().unwrap())
                //         .filter_map(|(retention_time, signal)| {
                //             Some(Bar::new(retention_time? as _, signal? as _))
                //         })
                //         .collect();
                //     let chart = BarChart::new(bars)
                //         .name(mass_to_charge.to_string())
                //         .stack_on(&charts.iter().collect::<Vec<_>>());
                //     charts.push(chart);
                // }
                // for chart in charts {
                //     ui.bar_chart(chart);
                // }

                // Bars
                // for (retention_time, (signal, peak)) in zip(retention_time, zip(signal, peak)) {
                //     // let mut offset = 0.0;
                //     if let (Some(retention_time), Some(signal), Some(peak)) =
                //         (retention_time, signal, peak)
                //     {
                //         if width > 10000.0 {
                //             let bar = Bar::new(retention_time as _, signal as _)
                //                 .name(retention_time.to_string());
                //             let chart = BarChart::new(vec![bar]).name(retention_time.to_string());
                //             // .color(color(retention_time as _));
                //             ui.bar_chart(chart);
                //         } else {
                //             let fields = peak.struct_().unwrap().fields();
                //             // tracing::error!(?peak);
                //             // let chart = BarChart::new(vec![bar]).name(retention_time.to_string());
                //         }
                //     }
                // }

                // for (retention_time, chunk) in &zip(retention_time, zip(mass_to_charge, signal))
                //     .chunk_by(|(retention_time, _)| *retention_time)
                // {
                //     let mut offset = 0.0;
                //     let mut bars = Vec::new();
                //     if let Some(retention_time) = retention_time {
                //         for (_, (mass_to_charge, signal)) in chunk {
                //             if let (Some(mass_to_charge), Some(signal)) = (mass_to_charge, signal) {
                //                 let bar = Bar::new(retention_time as _, signal as _)
                //                     .name(mass_to_charge.to_string())
                //                     .base_offset(offset as _);
                //                 bars.push(bar);
                //                 offset += signal;
                //             }
                //         }
                //         let chart = BarChart::new(bars).name(retention_time.to_string());
                //         // .color(color(retention_time as _));
                //         ui.bar_chart(chart);
                //     }
                // }

                // let mut iter = zip(retention_time.into_iter(), signal.into_iter());
                // while let Some((Some(x), Some(y))) = iter.next() {
                //     let bar = Bar::new(x as _, y as _).name("x");
                //     let chart = BarChart::new(vec![bar]).name(x).color(color(x as _));
                //     ui.bar_chart(chart);
                // }

                // for (key, values) in visualized {
                //     // Bars
                //     let mut offset = 0.0;
                //     let x = key.into_inner();
                //     for (name, value) in values {
                //         let mut y = value;
                //         if percent {
                //             y *= 100.0;
                //         }
                //         let bar = Bar::new(x, y).name(name).base_offset(offset);
                //         let chart = BarChart::new(vec![bar])
                //             .width(context.settings.visualization.width)
                //             .name(x)
                //             .color(color(x as _));
                //         ui.bar_chart(chart);
                //         offset += y;
                //     }
                //     // Text
                //     if context.settings.visualization.text.show
                //         && offset >= context.settings.visualization.text.min
                //     {
                //         let y = offset;
                //         let text = Text::new(
                //             PlotPoint::new(x, y),
                //             RichText::new(format!("{y:.p$}"))
                //                 .size(context.settings.visualization.text.size)
                //                 .heading(),
                //         )
                //         .name(x)
                //         .color(color(x as _))
                //         .anchor(Align2::CENTER_BOTTOM);
                //         ui.text(text);
                //     }
                // }
            });
        });
    }

    pub(super) fn grouped_by_retention_time(&self, ui: &mut Ui) {
        let frame = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<TableComputed>()
                .get(TableKey::new(&self.frame.data, &self.settings))
        });
        let plot_frame = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<PlotComputed>()
                .get(PlotKey::new(&frame, &self.settings))
        });
        let total_rows = frame.height();
        let retention_time = frame[RETENTION_TIME].f64().unwrap();
        let mass_spectrum = frame[MASS_SPECTRUM].list().unwrap();
        let mut plot = Plot::new("plot")
            .y_axis_formatter(move |y, _| round_to_decimals(y.value, 5).to_string());
        if self.settings.legend {
            let mut legend = Legend::default();
            // if let Some(visible) = self.settings.visible.take() {
            //     legend = if visible {
            //         legend.hidden_items(empty())
            //     } else {
            //         let hidden_items = retention_time
            //             .iter()
            //             .filter_map(|retention_time| Some(retention_time?.to_string()));
            //         legend.hidden_items(hidden_items)
            //     };
            // }
            plot = plot.legend(legend);
        }
        plot.show(ui, |ui| {
            let range_x = ui.plot_bounds().range_x();
            // let width = ui.plot_bounds().width();
            // tracing::error!(?width);

            // Bars
            let mut bars = IndexMap::<_, Vec<_>>::new();
            // for row_index in 0..total_rows {
            //     let retention_time = retention_time.get(row_index).unwrap();
            //     let mass_spectrum = mass_spectrum.get_as_series(row_index).unwrap();
            //     let mass_spectrum = mass_spectrum.struct_().unwrap();
            //     let mass_to_charge = mass_spectrum.field_by_name(MASS_TO_CHARGE).unwrap();
            //     let mass_to_charge = mass_to_charge.f32().unwrap();
            //     let signal = mass_spectrum.field_by_name(SIGNAL).unwrap();
            //     let signal = signal.u16().unwrap();
            //     for (mass_to_charge, signal) in zip(mass_to_charge, signal) {
            //         let mass_to_charge = mass_to_charge.unwrap();
            //         let signal = signal.unwrap();
            //         let bar =
            //             Bar::new(retention_time as _, signal as _).name(mass_to_charge.to_string());
            //         bars.push(bar);
            //     }
            // }
            for (retention_time, mass_spectrum) in zip(retention_time, mass_spectrum) {
                let retention_time = retention_time.unwrap();
                let mass_spectrum = mass_spectrum.unwrap();
                let mass_spectrum = mass_spectrum.struct_().unwrap();
                let mass_to_charge_series = mass_spectrum.field_by_name(MASS_TO_CHARGE).unwrap();
                let mass_to_charge = mass_to_charge_series.f32().unwrap();
                let signal_series = mass_spectrum.field_by_name(SIGNAL).unwrap();
                let signal = signal_series.u16().unwrap();
                for (mass_to_charge, signal) in zip(mass_to_charge, signal) {
                    let mass_to_charge = mass_to_charge.unwrap();
                    let signal = signal.unwrap();
                    let bar = Bar::new(retention_time, signal as _)
                        .name(mass_to_charge.to_string())
                        .width(0.01);
                    bars.entry(mass_to_charge.ord()).or_default().push(bar);
                }
            }
            for (mass_to_charge, bars) in bars {
                let index = mass_to_charge.0.round() as usize;
                let bar_chart = BarChart::new(index.to_string(), bars).color(color(index));
                ui.bar_chart(bar_chart);
            }
            // ui.line(Line::new("Median", series));
        });
    }
}
