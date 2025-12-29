use self::{events::Events, settings::Settings, windows::Windows};
use egui::{Context, Id};
use serde::{Deserialize, Serialize};

/// Pane state
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct State {
    pub(crate) events: Events,
    pub(crate) settings: Settings,
    pub(crate) windows: Windows,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            events: Events::new(),
            settings: Settings::new(),
            windows: Windows::new(),
        }
    }
}

impl State {
    pub(crate) fn load(ctx: &Context, id: Id) -> Self {
        ctx.data_mut(|data| data.get_persisted_mut_or_insert_with(id, Self::new).clone())
    }

    pub(crate) fn remove(self, ctx: &Context, id: Id) {
        ctx.data_mut(|data| {
            data.remove::<Self>(id);
        });
    }

    pub(crate) fn store(self, ctx: &Context, id: Id) {
        ctx.data_mut(|data| {
            data.insert_persisted(id, self);
        });
    }
}

pub(crate) mod events;
pub(crate) mod settings;
pub(crate) mod windows;
