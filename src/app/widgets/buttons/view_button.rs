use crate::app::states::pane::settings::View;
use egui::{Response, RichText, Ui, Widget};
use egui_l20n::prelude::*;

/// View button widget
#[derive(Debug)]
pub struct ViewButton<'a> {
    view: &'a mut View,
    size: Option<f32>,
}

impl<'a> ViewButton<'a> {
    pub fn new(view: &'a mut View) -> Self {
        Self { view, size: None }
    }

    pub fn size(self, size: f32) -> Self {
        Self {
            size: Some(size),
            ..self
        }
    }
}

impl Widget for ViewButton<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut atoms = RichText::new(self.view.icon());
        atoms = if let Some(size) = self.size {
            atoms.size(size)
        } else {
            atoms.heading()
        };
        ui.menu_button(atoms, |ui| {
            ui.selectable_value(self.view, View::Table, View::Table.text())
                .on_hover_localized(View::Table.hover_text());
            ui.selectable_value(self.view, View::Plot, View::Plot.text())
                .on_hover_localized(View::Plot.hover_text());
        })
        .response
    }
}
