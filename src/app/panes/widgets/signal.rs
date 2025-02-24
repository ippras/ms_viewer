use egui::{Response, Ui, Widget};
use polars::prelude::AnyValue;
use std::fmt::Display;

/// Signal widget
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SignalWidget<T> {
    pub(crate) value: Option<T>,
    pub(crate) precision: Option<usize>,
}

impl<T> SignalWidget<T> {
    pub(crate) const fn new(value: Option<T>) -> Self {
        Self {
            value,
            precision: None,
        }
    }

    pub(crate) fn precision(self, precision: Option<usize>) -> Self {
        Self { precision, ..self }
    }
}

impl<T: Display> Widget for SignalWidget<T> {
    fn ui(self, ui: &mut Ui) -> Response {
        if let Some(value) = self.value {
            let formated = if let Some(precision) = self.precision {
                format!("{value:.precision$}")
            } else {
                value.to_string()
            };
            ui.label(formated.to_string())
                .on_hover_text(value.to_string())
        } else {
            ui.label(AnyValue::Null.to_string())
        }
    }
}
