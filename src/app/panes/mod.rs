use self::{plot::PlotView, table::TableView};
use crate::{
    app::{
        ID_SOURCE,
        computers::{
            plot::{Computed as PlotComputed, Key as PlotKey},
            table::{Computed as TableComputed, Key as TableKey},
        },
        states::pane::{State, settings::View},
        widgets::buttons::{EditButton, ResetButton, ResizeButton, SettingsButton, ViewButton},
    },
    utils::hash::HashedMetaDataFrame,
};
use egui::{
    CentralPanel, CursorIcon, Direction, DragValue, Frame, Id, Layout, MenuBar, Response, RichText,
    ScrollArea, TextStyle, TopBottomPanel, Ui, Vec2, Widget as _, WidgetText, Window, util::hash,
    vec2,
};
use egui_l20n::prelude::*;
use egui_phosphor::regular::{SLIDERS_HORIZONTAL, TAG, X};
use egui_tiles::{TileId, UiResponse};
use metadata::egui::MetadataWidget;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, from_fn};

const MARGIN: Vec2 = vec2(4.0, 2.0);

/// Pane
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Pane {
    id: Option<Id>,
    pub(crate) frame: HashedMetaDataFrame,
}

impl Pane {
    pub(crate) fn new(frame: HashedMetaDataFrame) -> Self {
        Self { id: None, frame }
    }

    pub(crate) fn title(&self, separator: Option<&str>) -> String {
        self.frame
            .meta
            .format(separator.unwrap_or_default())
            .to_string()
    }

    pub(crate) fn id(&self) -> impl Display {
        from_fn(|f| {
            if let Some(id) = self.id {
                write!(f, "{id:?}-")?;
            }
            write!(f, "{}", hash(&self.frame))
        })
    }
}

impl Pane {
    pub(super) fn ui(
        &mut self,
        ui: &mut Ui,
        behavior: &mut Behavior,
        tile_id: TileId,
    ) -> UiResponse {
        let id = *self.id.get_or_insert_with(|| ui.next_auto_id());
        let mut state = State::load(ui.ctx(), id);
        let response = TopBottomPanel::top(ui.auto_id_with("Pane"))
            .show_inside(ui, |ui| {
                MenuBar::new()
                    .ui(ui, |ui| {
                        ScrollArea::horizontal()
                            .show(ui, |ui| {
                                ui.set_height(
                                    ui.text_style_height(&TextStyle::Heading) + 4.0 * MARGIN.y,
                                );
                                ui.visuals_mut().button_frame = false;
                                if ui.button(RichText::new(X).heading()).clicked() {
                                    behavior.close = Some(tile_id);
                                }
                                ui.separator();
                                self.top(ui, &mut state)
                            })
                            .inner
                    })
                    .inner
            })
            .inner;
        CentralPanel::default()
            .frame(Frame::central_panel(ui.style()))
            .show_inside(ui, |ui| {
                self.central(ui, &mut state);
                self.windows(ui, &mut state);
            });
        if behavior.close == Some(tile_id) {
            state.remove(ui.ctx(), id);
        } else {
            state.store(ui.ctx(), id);
        }
        if response.dragged() {
            UiResponse::DragStarted
        } else {
            UiResponse::None
        }
    }

    fn top(&mut self, ui: &mut Ui, state: &mut State) -> Response {
        let mut response = ui
            .heading(state.settings.view.icon())
            .on_hover_localized(state.settings.view.text());
        response |= ui.heading(self.title(Some(" ")));
        response = response
            .on_hover_text(self.id().to_string())
            .on_hover_ui(|ui| MetadataWidget::new(&self.frame.meta).show(ui))
            .on_hover_cursor(CursorIcon::Grab);
        ui.separator();
        ResetButton::new(&mut state.events.reset_table_state).ui(ui);
        ResizeButton::new(&mut state.settings.resizable).ui(ui);
        EditButton::new(&mut state.settings.edit).ui(ui);
        ViewButton::new(&mut state.settings.view).ui(ui);
        ui.separator();
        SettingsButton::new(&mut state.windows.open_settings).ui(ui);
        ui.separator();
        response
    }

    fn central(&mut self, ui: &mut Ui, state: &mut State) {
        if state.settings.edit {
            self.meta(ui);
        }
        self.data(ui, state);
    }

    fn meta(&mut self, ui: &mut Ui) {
        ui.style_mut().visuals.collapsing_header_frame = true;
        ui.collapsing(RichText::new(format!("{TAG} Metadata")).heading(), |ui| {
            MetadataWidget::new(&mut self.frame.meta)
                .with_writable(true)
                .show(ui);
        });
    }

    fn data(&mut self, ui: &mut Ui, state: &mut State) {
        match state.settings.view {
            View::Plot => {
                let data_frame = ui.memory_mut(|memory| {
                    memory
                        .caches
                        .cache::<PlotComputed>()
                        .get(PlotKey::new(&self.frame.data, &state.settings))
                });
            }
            View::Table => {
                let data_frame = ui.memory_mut(|memory| {
                    memory
                        .caches
                        .cache::<TableComputed>()
                        .get(TableKey::new(&self.frame.data, &state.settings))
                });
                TableView::new(&data_frame, state).show(ui);
            }
        }
    }
}

impl Pane {
    fn windows(&mut self, ui: &mut Ui, state: &mut State) {
        self.settings_window(ui, state);
    }

    fn settings_window(&mut self, ui: &mut Ui, state: &mut State) {
        Window::new(format!("{SLIDERS_HORIZONTAL} Configuration settings"))
            .id(ui.auto_id_with(ID_SOURCE).with("Settings"))
            .default_pos(ui.next_widget_position())
            .open(&mut state.windows.open_settings)
            .show(ui.ctx(), |ui| {
                state.settings.show(ui);
            });
    }
}

/// Behavior
#[derive(Debug)]
pub(crate) struct Behavior {
    pub(crate) close: Option<TileId>,
}

impl egui_tiles::Behavior<Pane> for Behavior {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> WidgetText {
        pane.title(Some(" ")).to_string().into()
    }

    fn pane_ui(&mut self, ui: &mut Ui, tile_id: TileId, pane: &mut Pane) -> UiResponse {
        pane.ui(ui, self, tile_id)
    }
}

// pub(crate) mod behavior;
pub(crate) mod plot;
pub(crate) mod table;
