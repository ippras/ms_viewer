use self::panes::{Behavior, Pane};
use crate::{
    app::{
        states::State,
        widgets::buttons::{
            GithubButton, GridButton, HorizontalButton, LeftPanelButton, ReactiveButton,
            ResetButton, SettingsButton, TabsButton, VerticalButton,
        },
    },
    r#const::EM_DASH,
    localization::ContextExt as _,
    utils::{TreeExt, hash::HashedDataFrame},
};
use data::Data;
use eframe::{APP_KEY, CreationContext, Storage, get_value, set_value};
use egui::{
    Align, Align2, CentralPanel, CollapsingHeader, Color32, Context, FontDefinitions, Frame, Id,
    LayerId, Layout, MenuBar, Order, RichText, ScrollArea, SidePanel, TextStyle, TopBottomPanel,
    Ui, Widget as _, Window, warn_if_debug_build,
};
use egui_ext::{DroppedFileExt, HoveredFileExt, LightDarkButton};
use egui_phosphor::{
    Variant, add_to_fonts,
    regular::{FLOPPY_DISK, SLIDERS_HORIZONTAL},
};
use egui_tiles::{Container, Tile, Tree};
use metadata::polars::MetaDataFrame;
use serde::{Deserialize, Serialize};
use std::{fmt::Write, str, time::Duration};
use tracing::info;

/// IEEE 754-2008
const MAX_PRECISION: usize = 16;
const _NOTIFICATIONS_DURATION: Duration = Duration::from_secs(15);
const ICON_SIZE: f32 = 32.0;
const ID_SOURCE: &str = "MS_VIEWER";

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct App {
    // // Panels
    // left_panel: bool,
    // Panes
    tree: Tree<Pane>,
    #[serde(skip)]
    behavior: Behavior,
}

impl Default for App {
    fn default() -> Self {
        Self {
            // left_panel: true,
            tree: Tree::empty("Tree"),
            behavior: Behavior { close: None },
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        let mut fonts = FontDefinitions::default();
        add_to_fonts(&mut fonts, Variant::Regular);
        cc.egui_ctx.set_fonts(fonts);
        cc.egui_ctx.set_localizations();

        // Default::default()
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        cc.storage
            .and_then(|storage| get_value(storage, APP_KEY))
            .unwrap_or_default()
    }

    fn drag_and_drop(&mut self, ctx: &Context) {
        // Preview hovering files
        if let Some(text) = ctx.input(|input| {
            (!input.raw.hovered_files.is_empty()).then(|| {
                let mut text = String::from("Dropping files:");
                for file in &input.raw.hovered_files {
                    write!(text, "\n{}", file.display()).ok();
                }
                text
            })
        }) {
            let painter =
                ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));
            let screen_rect = ctx.screen_rect();
            painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
            painter.text(
                screen_rect.center(),
                Align2::CENTER_CENTER,
                text,
                TextStyle::Heading.resolve(&ctx.style()),
                Color32::WHITE,
            );
        }
        // Parse dropped files
        if let Some(dropped_files) = ctx.input(|input| {
            (!input.raw.dropped_files.is_empty()).then_some(input.raw.dropped_files.clone())
        }) {
            info!(?dropped_files);
            for dropped_file in dropped_files {
                let bytes = dropped_file.bytes().unwrap();
                let frame: MetaDataFrame = ron::de::from_bytes(&bytes).unwrap();
                let data = HashedDataFrame::new(frame.data).unwrap();
                self.tree
                    .insert_pane(Pane::new(MetaDataFrame::new(frame.meta, data)));
            }
        }
    }
}

// Panels
impl App {
    fn container_ui(&self, ui: &mut Ui, container: &Container, depth: usize) {
        CollapsingHeader::new(format!(
            "{:?}[{}]",
            container.kind(),
            container.num_children(),
        ))
        .default_open(depth < 1)
        .show(ui, |ui| {
            for child in container.children() {
                match self.tree.tiles.get(*child) {
                    Some(Tile::Container(container)) => self.container_ui(ui, container, depth + 1),
                    Some(Tile::Pane(pane)) => self.pane_ui(ui, pane, depth + 1),
                    None => {
                        ui.label(EM_DASH);
                    }
                }
            }
        });
    }

    fn pane_ui(&self, ui: &mut Ui, pane: &Pane, depth: usize) {
        ui.label(pane.title(Some(" ")))
            .on_hover_text(pane.id().to_string());
    }

    fn panels(&mut self, ctx: &Context, state: &mut State) {
        self.top_panel(ctx, state);
        self.bottom_panel(ctx);
        self.left_panel(ctx, state);
        self.central_panel(ctx);
    }

    // Bottom panel
    fn bottom_panel(&mut self, ctx: &Context) {
        TopBottomPanel::bottom("BottomPanel").show(ctx, |ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                warn_if_debug_build(ui);
                ui.label(RichText::new(env!("CARGO_PKG_VERSION")).small());
                ui.separator();
            });
        });
    }

    // Central panel
    fn central_panel(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            self.tree.ui(&mut self.behavior, ui);
            if let Some(id) = self.behavior.close.take() {
                self.tree.tiles.remove(id);
            }
        });
    }

    // Left panel
    fn left_panel(&mut self, ctx: &Context, state: &mut State) {
        SidePanel::left("LeftPanel")
            .frame(Frame::side_top_panel(&ctx.style()))
            .resizable(true)
            .show_animated(ctx, state.settings.left_panel, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    if let Some(root) = self
                        .tree
                        .root
                        .and_then(|tile_id| self.tree.tiles.get_container(tile_id))
                    {
                        self.container_ui(ui, root, 0);
                    }
                });
            });
    }

    // Top panel
    fn top_panel(&mut self, ctx: &Context, state: &mut State) {
        TopBottomPanel::top("TopPanel").show(ctx, |ui| {
            MenuBar::new().ui(ui, |ui| {
                LeftPanelButton::new(&mut state.settings.left_panel)
                    .size(ICON_SIZE)
                    .ui(ui);
                ui.separator();
                ReactiveButton::new(&mut state.settings.reactive)
                    .size(ICON_SIZE)
                    .ui(ui);
                ui.separator();
                // Light/Dark
                ui.light_dark_button(ICON_SIZE);
                ui.separator();
                ResetButton::new(&mut state.settings.reset_state)
                    .size(ICON_SIZE)
                    .ui(ui);
                ui.separator();
                self.layouts(ui, state);
                ui.separator();
                SettingsButton::new(&mut state.windows.open_settings)
                    .size(ICON_SIZE)
                    .ui(ui);
                ui.separator();
                // Save
                ui.menu_button(FLOPPY_DISK, |ui| {
                    if ui.button("RON").clicked() {
                        for tile_id in self.tree.active_tiles() {
                            if let Some(tile) = self.tree.tiles.get(tile_id) {
                                match tile {
                                    Tile::Pane(pane) => {
                                        Data {
                                            frame: pane.frame.clone(),
                                        }
                                        .save("df.msv.ron")
                                        .unwrap();
                                    }
                                    Tile::Container(container) => {}
                                }
                            }
                        }
                    }
                });
                {
                    // for tile_id in self.tree.active_tiles() {
                    //     if let Some(root) = self.tree.root() {
                    //         self.tree.
                    //         match tile {
                    //             Tile::Pane(pane) => {
                    //                 pane.title();
                    //                 pane.data_frame();
                    //             }
                    //             Tile::Container(container) => todo!(),
                    //         }
                    //     }
                    // }
                    // if let Err(error) = self.data.save("df.utca.ron") {
                    //     error!(%error);
                    // }
                }
                ui.separator();
                GithubButton::new().size(ICON_SIZE).ui(ui);
                // Github.ui(ui);
                ui.separator();
                // ui.visuals_mut().button_frame = false;
                // global_dark_light_mode_switch(ui);
                // ui.separator();
                // if ui
                //     .add(Button::new(RichText::new("🗑")))
                //     .on_hover_text("Reset data")
                //     .clicked()
                // {
                //     *self = Default::default();
                // }
                // // Reset gui
                // if ui
                //     .add(Button::new(RichText::new("🔃")))
                //     .on_hover_text("Reset gui")
                //     .clicked()
                // {
                //     ui.with_visuals(|ui, _| ui.memory_mut(|memory| *memory = Default::default()));
                // }
                // // Organize windows
                // if ui
                //     .add(Button::new(RichText::new("▣")))
                //     .on_hover_text("Organize windows")
                //     .clicked()
                // {
                //     ui.ctx().memory_mut(|memory| memory.reset_areas());
                // }
                // ui.separator();
                // let mut central_tab = |tab| {
                //     let found = self.dock.find_tab(&tab);
                //     if ui
                //         .selectable_label(found.is_some(), tab.sign())
                //         .on_hover_text(tab.to_string())
                //         .clicked()
                //     {
                //         if let Some(index) = found {
                //             self.dock.remove_tab(index);
                //         } else {
                //             self.dock.push_to_focused_leaf(tab);
                //         }
                //     }
                // };
                // // Table
                // central_tab(CentralTab::Table);
                // // Plot
                // central_tab(CentralTab::Plot);
            });
        });
    }

    fn layouts(&mut self, ui: &mut Ui, state: &mut State) {
        VerticalButton::new(&mut state.settings.layout.container_kind)
            .size(ICON_SIZE)
            .ui(ui);
        HorizontalButton::new(&mut state.settings.layout.container_kind)
            .size(ICON_SIZE)
            .ui(ui);
        GridButton::new(&mut state.settings.layout.container_kind)
            .size(ICON_SIZE)
            .ui(ui);
        TabsButton::new(&mut state.settings.layout.container_kind)
            .size(ICON_SIZE)
            .ui(ui);
    }
}

// Windows
impl App {
    fn windows(&mut self, ctx: &Context, state: &mut State) {
        // self.about_window(ctx, state);
        self.settings_window(ctx, state);
    }

    // fn about_window(&mut self, ctx: &Context, state: &mut State) {
    //     Window::new(format!("{INFO} About"))
    //         .open(&mut state.windows.open_about)
    //         .show(ctx, |ui| About.ui(ui));
    // }

    fn settings_window(&mut self, ctx: &Context, state: &mut State) {
        Window::new(format!("{SLIDERS_HORIZONTAL} Settings"))
            .open(&mut state.windows.open_settings)
            .show(ctx, |ui| {
                state.settings.show(ui);
            });
    }
}

impl App {
    fn state(&mut self, ctx: &Context, state: &mut State) {
        if state.settings.reset_state {
            *self = Default::default();
            // Cache
            let caches = ctx.memory_mut(|memory| memory.caches.clone());
            ctx.memory_mut(|memory| {
                memory.caches = caches;
            });
            ctx.set_localizations();
            state.settings.reset_state = false;
        }
        if let Some(container_kind) = state.settings.layout.container_kind.take()
            && let Some(id) = self.tree.root
            && let Some(Tile::Container(container)) = self.tree.tiles.get_mut(id)
        {
            container.set_kind(container_kind);
        }
        if state.settings.reactive {
            ctx.request_repaint();
        }
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn Storage) {
        set_value(storage, APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let mut state = State::load(ctx, Id::new(ID_SOURCE));
        // Pre update
        self.panels(ctx, &mut state);
        self.windows(ctx, &mut state);
        // Post update
        self.drag_and_drop(ctx);
        self.state(ctx, &mut state);
        state.store(ctx, Id::new(ID_SOURCE));
    }
}

mod computers;
mod data;
mod panes;
mod states;
mod widgets;
