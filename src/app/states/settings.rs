use egui::Ui;
use egui_tiles::ContainerKind;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

/// Settings
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) layout: Layout,
    pub(crate) left_panel: bool,
    pub(crate) reactive: bool,
    pub(crate) reset_state: bool,
}

impl Settings {
    pub(crate) fn new() -> Self {
        Self {
            layout: Layout::new(),
            left_panel: false,
            reactive: false,
            reset_state: false,
        }
    }
}

impl Settings {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        //
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

/// Layout
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) struct Layout {
    pub(crate) container_kind: Option<ContainerKind>,
}

impl Layout {
    fn new() -> Self {
        Self {
            container_kind: None,
        }
    }
}

impl Hash for Layout {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.container_kind
            .map(|container_kind| container_kind as usize)
            .hash(state);
    }
}
