use crate::{
    app::{
        computers::{
            plot::{Computed as PlotComputed, Key as PlotKey, Value},
            table::{Computed as TableComputed, Key as TableKey},
        },
        states::pane::settings::{Settings, Sort},
    },
    r#const::*,
    utils::hash::{HashedDataFrame, HashedMetaDataFrame},
};
use egui::{
    Align2, Color32, RichText, Ui, Vec2,
    emath::{Float, OrderedFloat, round_to_decimals},
};
use egui_ext::color;
use egui_plot::{
    Bar, BarChart, HLine, Legend, Line, Plot, PlotMemory, PlotPoint, PlotPoints, Text,
};
use indexmap::IndexMap;
use itertools::Itertools;
use polars::{error::PolarsResult, frame::DataFrame};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    fmt::Write as _,
    iter::{empty, zip},
    rc::Rc,
};
use tracing::error;

/// Plot view
pub(crate) struct PlotView<'a> {
    pub(crate) data: Value,
    pub(crate) settings: &'a Settings,
}

impl<'a> PlotView<'a> {
    pub(crate) fn new(value: Value, settings: &'a Settings) -> Self {
        Self {
            data: value,
            settings,
        }
    }
}

impl PlotView<'_> {
    pub(super) fn show(self, ui: &mut Ui) {
        match self.settings.sort {
            Sort::RetentionTime if !self.settings.explode => self.grouped_by_retention_time(ui),
            // Sort::MassToCharge if !self.settings.explode => self.grouped_by_mass_to_charge(ui),
            _ => unimplemented!(),
        }
    }

    // pub(super) fn grouped_by_mass_to_charge(&self, ui: &mut Ui) {
    //     // let data_frame = ui.memory_mut(|memory| {
    //     //     memory
    //     //         .caches
    //     //         .cache::<TableComputed>()
    //     //         .get(TableKey::new(&self.data.data, &self.settings))
    //     // });
    //     // let points = data_frame.height();
    //     let mass_to_charge = data_frame[MASS_TO_CHARGE].f32().unwrap();
    //     let retention_time = data_frame[RETENTION_TIME].list().unwrap();
    //     let signal = data_frame[SIGNAL].list().unwrap();
    //     ui.vertical_centered_justified(|ui| {
    //         // let id = ui.make_persistent_id("plot");
    //         // let plot_memory = PlotMemory::load(ui.ctx(), id);
    //         let mut plot = Plot::new("plot")
    //             .y_axis_formatter(move |y, _| round_to_decimals(y.value, 5).to_string());
    //         if self.settings.plot.legend {
    //             let mut legend = Legend::default();
    //             // if let Some(visible) = self.settings.visible.take() {
    //             //     legend = if visible {
    //             //         legend.hidden_items(empty())
    //             //     } else {
    //             //         let hidden_items = mass_to_charge
    //             //             .iter()
    //             //             .filter_map(|mass_to_charge| Some(mass_to_charge?.to_string()));
    //             //         legend.hidden_items(hidden_items)
    //             //     };
    //             // }
    //             plot = plot.legend(legend);
    //         }
    //         plot.show(ui, |ui| {
    //             // let bounds = ui.plot_bounds().range_x();
    //             // let width = ui.plot_bounds().width();
    //             // tracing::error!(?width);

    //             // Lines
    //             for (mass_to_charge, retention_time, signal) in
    //                 zip(mass_to_charge, zip(retention_time, signal)).filter_map(
    //                     |(mass_to_charge, (retention_time, signal))| {
    //                         Some((mass_to_charge?, retention_time?, signal?))
    //                     },
    //                 )
    //             {
    //                 let line = Line::new(
    //                     mass_to_charge.to_string(),
    //                     PlotPoints::from_iter(
    //                         zip(retention_time.i32().unwrap(), signal.u16().unwrap()).filter_map(
    //                             |(retention_time, signal)| {
    //                                 Some([retention_time? as _, signal? as _])
    //                             },
    //                         ),
    //                     ),
    //                 );
    //                 ui.line(line);
    //             }

    //             // // Bars
    //             // for (mass_to_charge, retention_time, signal) in
    //             //     zip(mass_to_charge, zip(retention_time, signal)).filter_map(
    //             //         |(mass_to_charge, (retention_time, signal))| {
    //             //             Some((mass_to_charge?, retention_time?, signal?))
    //             //         },
    //             //     )
    //             // {
    //             //     let bars = zip(retention_time.i32().unwrap(), signal.u16().unwrap())
    //             //         .filter_map(|(retention_time, signal)| {
    //             //             Some(Bar::new(retention_time? as _, signal? as _))
    //             //         })
    //             //         .collect();
    //             //     let chart = BarChart::new(bars).name(mass_to_charge.to_string());
    //             //     ui.bar_chart(chart);
    //             // }

    //             // let mut charts = Vec::new();
    //             // for (mass_to_charge, retention_time, signal) in
    //             //     zip(mass_to_charge, zip(retention_time, signal)).filter_map(
    //             //         |(mass_to_charge, (retention_time, signal))| {
    //             //             Some((mass_to_charge?, retention_time?, signal?))
    //             //         },
    //             //     )
    //             // {
    //             //     let bars = zip(retention_time.i32().unwrap(), signal.().unwrap())
    //             //         .filter_map(|(retention_time, signal)| {
    //             //             Some(Bar::new(retention_time? as _, signal? as _))
    //             //         })
    //             //         .collect();
    //             //     let chart = BarChart::new(bars)
    //             //         .name(mass_to_charge.to_string())
    //             //         .stack_on(&charts.iter().collect::<Vec<_>>());
    //             //     charts.push(chart);
    //             // }
    //             // for chart in charts {
    //             //     ui.bar_chart(chart);
    //             // }

    //             // Bars
    //             // for (retention_time, (signal, peak)) in zip(retention_time, zip(signal, peak)) {
    //             //     // let mut offset = 0.0;
    //             //     if let (Some(retention_time), Some(signal), Some(peak)) =
    //             //         (retention_time, signal, peak)
    //             //     {
    //             //         if width > 10000.0 {
    //             //             let bar = Bar::new(retention_time as _, signal as _)
    //             //                 .name(retention_time.to_string());
    //             //             let chart = BarChart::new(vec![bar]).name(retention_time.to_string());
    //             //             // .color(color(retention_time as _));
    //             //             ui.bar_chart(chart);
    //             //         } else {
    //             //             let fields = peak.struct_().unwrap().fields();
    //             //             // tracing::error!(?peak);
    //             //             // let chart = BarChart::new(vec![bar]).name(retention_time.to_string());
    //             //         }
    //             //     }
    //             // }

    //             // for (retention_time, chunk) in &zip(retention_time, zip(mass_to_charge, signal))
    //             //     .chunk_by(|(retention_time, _)| *retention_time)
    //             // {
    //             //     let mut offset = 0.0;
    //             //     let mut bars = Vec::new();
    //             //     if let Some(retention_time) = retention_time {
    //             //         for (_, (mass_to_charge, signal)) in chunk {
    //             //             if let (Some(mass_to_charge), Some(signal)) = (mass_to_charge, signal) {
    //             //                 let bar = Bar::new(retention_time as _, signal as _)
    //             //                     .name(mass_to_charge.to_string())
    //             //                     .base_offset(offset as _);
    //             //                 bars.push(bar);
    //             //                 offset += signal;
    //             //             }
    //             //         }
    //             //         let chart = BarChart::new(bars).name(retention_time.to_string());
    //             //         // .color(color(retention_time as _));
    //             //         ui.bar_chart(chart);
    //             //     }
    //             // }

    //             // let mut iter = zip(retention_time.into_iter(), signal.into_iter());
    //             // while let Some((Some(x), Some(y))) = iter.next() {
    //             //     let bar = Bar::new(x as _, y as _).name("x");
    //             //     let chart = BarChart::new(vec![bar]).name(x).color(color(x as _));
    //             //     ui.bar_chart(chart);
    //             // }

    //             // for (key, values) in visualized {
    //             //     // Bars
    //             //     let mut offset = 0.0;
    //             //     let x = key.into_inner();
    //             //     for (name, value) in values {
    //             //         let mut y = value;
    //             //         if percent {
    //             //             y *= 100.0;
    //             //         }
    //             //         let bar = Bar::new(x, y).name(name).base_offset(offset);
    //             //         let chart = BarChart::new(vec![bar])
    //             //             .width(context.settings.visualization.width)
    //             //             .name(x)
    //             //             .color(color(x as _));
    //             //         ui.bar_chart(chart);
    //             //         offset += y;
    //             //     }
    //             //     // Text
    //             //     if context.settings.visualization.text.show
    //             //         && offset >= context.settings.visualization.text.min
    //             //     {
    //             //         let y = offset;
    //             //         let text = Text::new(
    //             //             PlotPoint::new(x, y),
    //             //             RichText::new(format!("{y:.p$}"))
    //             //                 .size(context.settings.visualization.text.size)
    //             //                 .heading(),
    //             //         )
    //             //         .name(x)
    //             //         .color(color(x as _))
    //             //         .anchor(Align2::CENTER_BOTTOM);
    //             //         ui.text(text);
    //             //     }
    //             // }
    //         });
    //     });
    // }

    pub(super) fn grouped_by_retention_time(self, ui: &mut Ui) {
        // let frame = ui.memory_mut(|memory| {
        //     memory
        //         .caches
        //         .cache::<TableComputed>()
        //         .get(TableKey::new(&self.frame.data, &self.settings))
        // });
        // let value = ui.memory_mut(|memory| {
        //     memory
        //         .caches
        //         .cache::<PlotComputed>()
        //         .get(PlotKey::new(&frame, &self.settings))
        // });
        let value = self.data;
        let mut plot = Plot::new("Plot").label_formatter(|name, value| {
            if !name.is_empty() {
                format!("{}\nx: {}\ny: {}", name, value.x, value.y)
            } else {
                format!("x: {}\ny: {}", value.x, value.y)
                // "".to_owned()
            }
        });
        // .label_formatter(move |name, PlotPoint { x, y }| {
        //             let mut label = String::new();
        //             if !name.is_empty() {
        //                 _ = writeln!(&mut label, "{name}");
        //             }
        //             if let Some(values) = points.get(&IndexKey(PlotPoint::new(*x, *y))) {
        //                 _ = writeln!(
        //                     &mut label,
        //                     "{onset_temperature} = {}",
        //                     values
        //                         .iter()
        //                         .map(|value| value.onset_temperature)
        //                         .format(","),
        //                 );
        //                 _ = writeln!(
        //                     &mut label,
        //                     "{temperature_step} = {}",
        //                     values
        //                         .iter()
        //                         .map(|value| value.temperature_step)
        //                         .format(","),
        //                 );
        //             }
        //             let precision = self.settings.precision;
        //             _ = writeln!(&mut label, "{retention_time} = {x:.precision$}");
        //             _ = write!(&mut label, "{equivalent_chain_length} = {y:.precision$}");
        //             label
        //         });
        // .y_axis_formatter(move |y, _| round_to_decimals(y.value, 5).to_string());
        if self.settings.plot.legend {
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
        let plot_response = plot.show(ui, |ui| {
            let range_x = ui.plot_bounds().range_x();
            // let width = ui.plot_bounds().width();
            // tracing::error!(?width);

            // Bar chart
            let mass_spectrums = Rc::new(value.mass_spectrums);
            for (mass_to_charge, bars) in value.bars {
                let mass_spectrums = mass_spectrums.clone();
                let index = mass_to_charge.0.round() as usize;
                // bar_chart = if !self.settings.threshold.filter {
                //     bar_chart.color(Color32::GRAY)
                // } else {
                //     bar_chart.color(color(index))
                // };
                let bar_chart = BarChart::new("Bar chart", bars)
                    .color(color(index))
                    .element_formatter(Box::new(move |bar, _bar_chart| {
                        let mut label = String::new();
                        _ = writeln!(&mut label, "Retention time (x): {}", bar.argument);
                        _ = writeln!(&mut label, "Signal (y): {}", bar.value);
                        _ = writeln!(&mut label, "Mass to charge: {}", bar.name);
                        let mass_spectrum = &mass_spectrums[&bar.argument.ord()];
                        let Some((position, _)) =
                            mass_spectrum
                                .iter()
                                .find_position(|mass_to_charge_and_signal| {
                                    mass_to_charge_and_signal.0 == mass_to_charge.0
                                })
                        else {
                            return label;
                        };
                        _ = writeln!(&mut label, "Mass spectrum:");
                        let start = position.saturating_sub(10);
                        let end = position.saturating_add(10).min(mass_spectrum.len());
                        for (mass_to_charge, signal) in mass_spectrum[start..end].iter().rev() {
                            _ = writeln!(
                                &mut label,
                                "\tMass to charge: {mass_to_charge}; Signal: {signal}"
                            );
                        }
                        label
                    }));
                ui.bar_chart(bar_chart);
            }
            // Mean
            if let Some(mean) = value.mean {
                ui.hline(HLine::new("Mean", mean.0));
            }
            // Median
            if let Some(median) = value.median {
                ui.hline(HLine::new("Median", median.0));
            }
            // Rolling mean
            if !value.rolling_mean.is_empty() {
                ui.line(Line::new("RollingMean", value.rolling_mean));
            }
            if !value.rolling_median.is_empty() {
                ui.line(Line::new("RollingMedian", value.rolling_median));
            }
        });
        plot_response.response.context_menu(|ui| {
            if ui.button("RetentionTime").clicked() {
                let id = plot_response.hovered_plot_item;
                println!("RetentionTime: ");
            }
        });
    }
}
