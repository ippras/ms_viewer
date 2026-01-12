use crate::utils::{hash::HashedMetaDataFrame, spawn::spawn};
use anyhow::{Context as _, Error, Result};
use egui::{
    Context, Id, PopupCloseBehavior, Response, RichText, ScrollArea, Ui, Widget,
    containers::menu::{MenuButton, MenuConfig},
};
use egui_phosphor::regular::CLOUD_ARROW_DOWN;
use ehttp::{Request, fetch_async};
use std::borrow::Cow;
use tracing::{instrument, trace};
use url::Url;
use urlencoding::decode;

/// Github button widget
pub struct GithubButton {
    size: Option<f32>,
}

impl GithubButton {
    pub fn new() -> Self {
        Self { size: None }
    }

    #[allow(unused)]
    pub fn size(self, size: f32) -> Self {
        Self {
            size: Some(size),
            ..self
        }
    }
}

impl Widget for GithubButton {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut atoms = RichText::new(CLOUD_ARROW_DOWN);
        if let Some(size) = self.size {
            atoms = atoms.size(size)
        }
        MenuButton::new(atoms)
            .config(MenuConfig::new().close_behavior(PopupCloseBehavior::CloseOnClickOutside))
            .ui(ui, |ui| {
                ScrollArea::new([false, true]).show(ui, content);
            })
            .0
    }
}

/// Content
fn content(ui: &mut Ui) {
    // IPPRAS
    ui.hyperlink_to(RichText::new("IPPRAS").heading(), "https://ippras.ru");
    ScrollArea::new([false, true]).show(ui, |ui| {
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=40;TemperatureStep=1}[1].2024-7-31.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=1}[1].2024-9-04.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=1}[2].2024-9-18.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=1}[3].2024-7-31.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=2}[1].2024-9-04.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=2}[2].2024-9-17.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=2}[3].2024-7-31.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=3}[1].2024-9-04.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=3}[2].2024-9-18.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=3}[3].2024-7-31.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=4}[1].2024-9-05.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=4}[2].2024-9-18.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=4}[3].2024-7-31.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=5}[1].2024-9-05.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=5}[2].2024-9-18.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=5}[3].2024-7-31.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=6}[1].2024-9-04.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=6}[2].2024-9-18.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=6}[3].2024-7-31.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=7}[1].2024-9-04.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=7}[2].2024-9-18.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=7}[3].2024-7-31.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=8}[1].2024-9-04.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=8}[2].2024-9-18.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=8}[3].2024-7-31.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=9}[1].2024-9-04.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=9}[2].2024-9-18.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=9}[3].2024-7-31.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=10}[1].2024-9-04.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=10}[2].2024-9-18.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=60;TemperatureStep=10}[3].2024-7-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=1}[1].2024-9-03.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=1}[2].2024-9-17.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=1}[3].2024-7-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=2}[1].2024-9-03.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=2}[2].2024-9-17.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=2}[3].2024-7-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=3}[1].2024-9-04.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=3}[2].2024-9-17.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=3}[3].2024-7-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=4}[1].2024-9-04.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=4}[2].2024-9-17.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=4}[3].2024-7-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=5}[1].2024-9-04.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=5}[2].2024-9-17.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=5}[3].2024-7-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=6}[1].2024-9-04.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=6}[2].2024-9-17.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=6}[3].2024-7-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=7}[1].2024-9-04.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=7}[2].2024-9-17.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=7}[3].2024-7-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=8}[1].2024-9-17.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=8}[2].2024-7-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=8}[3].2024-9-03.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=9}[1].2024-9-17.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=9}[2].2024-7-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=9}[3].2024-9-03.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=10}[1].2024-9-17.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=10}[2].2024-7-29.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=70;TemperatureStep=10}[3].2024-9-03.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=1}[1].2024-9-16.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=1}[2].2024-7-29.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=1}[3].2024-9-03.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=2}[1].2024-9-16.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=2}[2].2024-7-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=2}[3].2024-9-03.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=3}[1].2024-9-17.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=3}[2].2024-7-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=3}[3].2024-9-03.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=4}[1].2024-9-17.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=4}[2].2024-7-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=4}[3].2024-9-03.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=5}[1].2024-9-17.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=5}[2].2024-7-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=5}[3].2024-9-03.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=6}[1].2024-9-16.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=6}[2].2024-7-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=6}[3].2024-9-03.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=7}[1].2024-9-16.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=7}[2].2024-7-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=7}[3].2024-9-03.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=8}[1].2024-9-16.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=8}[2].2024-7-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=8}[3].2024-9-03.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=9}[1].2024-9-16.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=9}[2].2024-7-29.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=9}[3].2024-9-03.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=10}[1].2024-9-16.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=10}[2].2024-7-29.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=80;TemperatureStep=10}[3].2024-9-02.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=1}[1].2024-9-16.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=1}[2].2024-7-29.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=1}[3].2024-9-02.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=2}[1].2024-9-16.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=2}[2].2024-7-29.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=2}[3].2024-9-02.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=3}[1].2024-9-16.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=3}[2].2024-7-29.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=3}[3].2024-9-02.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=4}[1].2024-9-16.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=4}[2].2024-7-29.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=4}[3].2024-9-02.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=5}[1].2024-9-16.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=5}[2].2024-7-29.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=5}[3].2024-9-02.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=6}[1].2024-9-16.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=6}[2].2024-7-29.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=6}[3].2024-9-02.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=7}[1].2024-9-16.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=7}[2].2024-7-29.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=7}[3].2024-9-02.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=8}[1].2024-9-16.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=8}[2].2024-7-29.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=8}[3].2024-9-02.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=9}[1].2024-9-16.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=9}[2].2024-7-29.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=9}[3].2024-9-02.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=10}[1].2024-9-16.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=10}[2].2024-7-26.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=90;TemperatureStep=10}[3].2024-8-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=1}[1].2024-9-13.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=1}[2].2024-7-26.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=1}[3].2024-9-02.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=2}[1].2024-9-13.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=2}[2].2024-7-26.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=2}[3].2024-9-02.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=3}[1].2024-9-13.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=3}[2].2024-7-26.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=3}[3].2024-9-02.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=4}[1].2024-9-13.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=4}[2].2024-7-26.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=4}[3].2024-8-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=5}[1].2024-9-13.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=5}[2].2024-7-26.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=5}[3].2024-8-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=6}[1].2024-9-13.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=6}[2].2024-7-26.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=6}[3].2024-8-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=7}[1].2024-9-13.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=7}[2].2024-7-26.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=7}[3].2024-8-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=8}[1].2024-9-13.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=8}[2].2024-7-26.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=8}[3].2024-8-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=9}[1].2024-9-13.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=9}[2].2024-7-26.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=9}[3].2024-8-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=10}[1].2024-9-13.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=10}[2].2024-7-25.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=100;TemperatureStep=10}[3].2024-8-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=1}[1].2024-9-12.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=1}[2].2024-9-24.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=1}[3].2024-7-25.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=2}[1].2024-7-25.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=2}[2].2024-8-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=2}[3].2024-9-12.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=3}[1].2024-7-26.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=3}[2].2024-8-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=3}[3].2024-9-13.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=4}[1].2024-7-26.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=4}[2].2024-8-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=4}[3].2024-9-12.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=5}[1].2024-7-26.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=5}[2].2024-8-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=5}[3].2024-9-12.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=6}[1].2024-7-26.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=6}[2].2024-8-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=6}[3].2024-9-12.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=7}[1].2024-7-26.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=7}[2].2024-8-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=7}[3].2024-9-12.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=8}[1].2024-7-26.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=8}[2].2024-8-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=8}[3].2024-9-12.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=9}[1].2024-7-26.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=9}[2].2024-8-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=9}[3].2024-9-12.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=10}[1].2024-9-24.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=10}[2].2024-7-25.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=110;TemperatureStep=10}[3].2024-8-30.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=1}[1].2024-9-12.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=1}[2].2024-9-24.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=1}[3].2024-7-08.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=2}[1].2024-8-07.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=2}[2].2024-9-10.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=2}[3].2024-9-23.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=3}[1].2024-8-07.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=3}[2].2024-9-12.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=3}[3].2024-9-23.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=4}[1].2024-8-20.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=4}[2].2024-9-11.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=4}[3].2024-9-24.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=5}[1].2024-8-07.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=5}[2].2024-9-10.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=5}[3].2024-9-24.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=6}[1].2024-8-07.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=6}[2].2024-9-10.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=6}[3].2024-9-24.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=7}[1].2024-8-07.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=7}[2].2024-9-10.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=7}[3].2024-9-24.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=8}[1].2024-8-07.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=8}[2].2024-9-10.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=8}[3].2024-9-24.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=9}[1].2024-8-07.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=9}[2].2024-9-10.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=9}[3].2024-9-24.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=10}[1].2024-8-07.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=10}[2].2024-9-10.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=120;TemperatureStep=10}[3].2024-9-23.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=1}[1].2024-8-07.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=1}[2].2024-9-10.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=1}[3].2024-9-23.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=2}[1].2024-8-06.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=2}[2].2024-9-06.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=2}[3].2024-9-20.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=3}[1].2024-8-06.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=3}[2].2024-9-09.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=3}[3].2024-9-23.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=4}[1].2024-8-07.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=4}[2].2024-9-09.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=4}[3].2024-9-23.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=5}[1].2024-8-07.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=5}[2].2024-9-09.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=5}[3].2024-9-23.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=6}[1].2024-8-06.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=6}[2].2024-9-09.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=6}[3].2024-9-23.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=7}[1].2024-8-06.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=7}[2].2024-9-09.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=7}[3].2024-9-20.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=8}[1].2024-8-06.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=8}[2].2024-9-09.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=8}[3].2024-9-20.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=9}[1].2024-8-06.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=9}[2].2024-9-09.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=9}[3].2024-9-20.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=10}[1].2024-9-23.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=10}[2].2024-8-06.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=130;TemperatureStep=10}[3].2024-9-09.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=1}[1].2024-9-20.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=1}[2].2024-8-06.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=1}[3].2024-9-09.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=2}[1].2024-9-20.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=2}[2].2024-8-01.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=2}[3].2024-9-06.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=3}[1].2024-9-19.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=3}[2].2024-8-01.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=3}[3].2024-9-06.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=4}[1].2024-9-19.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=4}[2].2024-8-05.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=4}[3].2024-9-06.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=5}[1].2024-9-20.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=5}[2].2024-8-06.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=5}[3].2024-9-06.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=6}[1].2024-9-20.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=6}[2].2024-8-06.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=6}[3].2024-9-06.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=7}[1].2024-9-20.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=7}[2].2024-8-06.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=7}[3].2024-9-06.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=8}[1].2024-9-20.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=8}[2].2024-8-06.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=8}[3].2024-9-06.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=9}[1].2024-9-20.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=9}[2].2024-8-06.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=9}[3].2024-9-06.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=10}[1].2024-9-19.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=10}[2].2024-8-01.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=140;TemperatureStep=10}[3].2024-9-06.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=1}[1].2024-9-19.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=1}[2].2024-8-01.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=1}[3].2024-9-06.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=2}[1].2024-9-19.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=2}[2].2024-7-31.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=2}[3].2024-9-05.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=3}[1].2024-9-18.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=3}[2].2024-7-31.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=3}[3].2024-9-05.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=4}[1].2024-9-18.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=4}[2].2024-8-01.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=4}[3].2024-9-05.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=5}[1].2024-9-19.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=5}[2].2024-8-01.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=5}[3].2024-9-05.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=6}[1].2024-9-19.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=6}[2].2024-8-01.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=6}[3].2024-9-05.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=7}[1].2024-9-19.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=7}[2].2024-8-01.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=7}[3].2024-9-05.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=8}[1].2024-9-19.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=8}[2].2024-8-01.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=8}[3].2024-9-05.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=8}[4].2024-9-19.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=9}[1].2024-8-01.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=9}[2].2024-9-05.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=9}[3].2024-9-19.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=9}[4].2024-8-01.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=10}[1].2024-9-05.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=10}[2].2024-9-18.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=10}[3].2024-7-31.ron");
        _ = preset(ui, "https://raw.githubusercontent.com/ippras/_cpft/data/ConstantFlow{InitialTemperature=150;TemperatureStep=10}[4].2024-9-05.ron");
    });
}

/// Preset
#[instrument(skip(ui), err)]
fn preset(ui: &mut Ui, input: &str) -> Result<()> {
    let url = Url::parse(input)?;
    let (name, date) = parse(&url)?;
    if ui
        .button(format!("{CLOUD_ARROW_DOWN} {name} {date}"))
        .clicked()
    {
        load(ui.ctx(), url);
    }
    Ok(())
}

/// Parse preset url
fn parse<'a>(url: &'a Url) -> Result<(Cow<'a, str>, &'a str)> {
    let segment = url
        .path_segments()
        .context("Preset get path segments")?
        .last()
        .context("Preset get last path segment")?;
    let input = segment.trim_end_matches(".ron");
    let (name, date) = input
        .rsplit_once(".")
        .context("Preset parse name and date")?;
    Ok((decode(name)?, date))
}

fn load(ctx: &Context, url: Url) {
    let ctx = ctx.clone();
    _ = spawn(async move {
        if let Ok(frame) = try_load(&url).await {
            trace!(?frame);
            ctx.data_mut(|data| data.insert_temp(Id::new("Data"), frame));
        }
    });
}

#[instrument(err)]
async fn try_load(url: &Url) -> Result<HashedMetaDataFrame> {
    let request = Request::get(url);
    let response = fetch_async(request).await.map_err(Error::msg)?;
    let text = response.text().context("Try load get response text")?;
    trace!(?text);
    Ok(ron::de::from_str(text)?)
}
