use self::panes::{Pane, behavior::Behavior};
use crate::utils::{
    TreeExt,
    hash::{HashedDataFrame, HashedMetaDataFrame},
};
use anyhow::Result;
use data::{Data, Format};
use eframe::{APP_KEY, CreationContext, Storage, get_value, set_value};
use egui::{
    Align, Align2, CentralPanel, Color32, Context, DroppedFile, FontDefinitions, Frame, Id,
    LayerId, Layout, Order, RichText, ScrollArea, SidePanel, TextStyle, TopBottomPanel, menu::bar,
    warn_if_debug_build,
};
use egui_ext::{DroppedFileExt, HoveredFileExt, LightDarkButton};
use egui_phosphor::{
    Variant, add_to_fonts,
    regular::{
        ARROWS_CLOCKWISE, FLOPPY_DISK, GRID_FOUR, ROCKET, SIDEBAR_SIMPLE, SQUARE_SPLIT_HORIZONTAL,
        SQUARE_SPLIT_VERTICAL, TABLE, TABS, TRASH,
    },
};
use egui_tiles::{ContainerKind, Tile, Tree};
use metadata::polars::MetaDataFrame;
use panes::table::TablePane;
use polars::frame::DataFrame;
use serde::{Deserialize, Serialize};
use std::{fmt::Write, io::Cursor, str, time::Duration};
use tracing::{error, info, trace};

macro icon($icon:expr) {
    RichText::new($icon).size(SIZE)
}

macro localize($text:literal) {
    $text
}

/// IEEE 754-2008
const MAX_PRECISION: usize = 16;
const _NOTIFICATIONS_DURATION: Duration = Duration::from_secs(15);
const SIZE: f32 = 32.0;

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct App {
    reactive: bool,
    // Panels
    left_panel: bool,
    // Panes
    tree: Tree<Pane>,
    behavior: Behavior,
}

impl Default for App {
    fn default() -> Self {
        Self {
            reactive: true,
            left_panel: true,
            tree: Tree::empty("tree"),
            behavior: Default::default(),
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
                // let data_frame: DataFrame = match dropped_file.extension() {
                //     Some("bin") => bincode::deserialize(&fs::read(&args.path)?)?,
                //     Some("ron") => ron::de::from_str(&fs::read_to_string(&args.path)?)?,
                //     _ => panic!("unsupported input file extension"),
                // };
                // dropped_file.extension();
                let bytes = dropped_file.bytes().unwrap();
                let frame: MetaDataFrame = ron::de::from_bytes(&bytes).unwrap();
                let data = HashedDataFrame::new(frame.data).unwrap();
                self.tree.insert_pane(Pane::Table(TablePane {
                    frame: MetaDataFrame::new(frame.meta, data),
                    settings: Default::default(),
                }));
                // match bin(&dropped_file) {
                //     Ok(data_frame) => {
                //         trace!(?data_frame);
                //         self.tree.insert_pane(Pane::Table(TablePane {
                //             data_frame,
                //             settings: Default::default(),
                //         }));
                //     }
                //     Err(error) => {
                //         error!(%error);
                //         // self.toasts
                //         //     .error(format!("{}: {error}", dropped.display()))
                //         //     .set_closable(true)
                //         //     .set_duration(Some(NOTIFICATIONS_DURATION));
                //         continue;
                //     }
                // };
            }
        }
    }
}

impl App {
    fn panels(&mut self, ctx: &Context) {
        self.top_panel(ctx);
        self.bottom_panel(ctx);
        self.left_panel(ctx);
        self.central_panel(ctx);
    }

    // Bottom panel
    fn bottom_panel(&mut self, ctx: &Context) {
        TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
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
    fn left_panel(&mut self, ctx: &Context) {
        SidePanel::left("left_panel")
            .frame(Frame::side_top_panel(&ctx.style()))
            .resizable(true)
            .show_animated(ctx, self.left_panel, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    self.behavior.settings(ui, &mut self.tree);
                    ui.separator();
                });
            });
    }

    // Top panel
    fn top_panel(&mut self, ctx: &Context) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            bar(ui, |ui| {
                // Left panel
                ui.toggle_value(&mut self.left_panel, icon!(SIDEBAR_SIMPLE))
                    .on_hover_text(localize!("left_panel"));
                ui.separator();
                ui.light_dark_button(SIZE);
                ui.separator();
                ui.toggle_value(&mut self.reactive, icon!(ROCKET))
                    .on_hover_text("reactive")
                    .on_hover_text(localize!("reactive_description_enabled"))
                    .on_disabled_hover_text(localize!("reactive_description_disabled"));
                ui.separator();
                if ui
                    .button(icon!(TRASH))
                    .on_hover_text(localize!("reset_application"))
                    .clicked()
                {
                    *self = Default::default();
                }
                ui.separator();
                if ui
                    .button(icon!(ARROWS_CLOCKWISE))
                    .on_hover_text(localize!("reset_gui"))
                    .clicked()
                {
                    ui.memory_mut(|memory| *memory = Default::default());
                }
                ui.separator();
                if ui
                    .button(icon!(SQUARE_SPLIT_VERTICAL))
                    .on_hover_text(localize!("vertical"))
                    .clicked()
                {
                    if let Some(id) = self.tree.root {
                        if let Some(Tile::Container(container)) = self.tree.tiles.get_mut(id) {
                            container.set_kind(ContainerKind::Vertical);
                        }
                    }
                }
                if ui
                    .button(icon!(SQUARE_SPLIT_HORIZONTAL))
                    .on_hover_text(localize!("horizontal"))
                    .clicked()
                {
                    if let Some(id) = self.tree.root {
                        if let Some(Tile::Container(container)) = self.tree.tiles.get_mut(id) {
                            container.set_kind(ContainerKind::Horizontal);
                        }
                    }
                }
                if ui
                    .button(icon!(GRID_FOUR))
                    .on_hover_text(localize!("grid"))
                    .clicked()
                {
                    if let Some(id) = self.tree.root {
                        if let Some(Tile::Container(container)) = self.tree.tiles.get_mut(id) {
                            container.set_kind(ContainerKind::Grid);
                        }
                    }
                }
                if ui
                    .button(icon!(TABS))
                    .on_hover_text(localize!("tabs"))
                    .clicked()
                {
                    if let Some(id) = self.tree.root {
                        if let Some(Tile::Container(container)) = self.tree.tiles.get_mut(id) {
                            container.set_kind(ContainerKind::Tabs);
                        }
                    }
                }
                //
                ui.separator();
                // Save
                ui.menu_button(icon!(FLOPPY_DISK), |ui| {
                    if ui.button("RON").clicked() {
                        for tile_id in self.tree.active_tiles() {
                            if let Some(tile) = self.tree.tiles.get(tile_id) {
                                match tile {
                                    Tile::Pane(pane) => {
                                        Data {
                                            frame: pane.frame().clone(),
                                        }
                                        .save("df.msv.ron", Format::Ron)
                                        .unwrap();
                                    }
                                    Tile::Container(container) => {}
                                }
                            }
                        }
                    }
                    if ui.button("BIN").clicked() {
                        for tile_id in self.tree.active_tiles() {
                            if let Some(tile) = self.tree.tiles.get(tile_id) {
                                match tile {
                                    Tile::Pane(pane) => {
                                        Data {
                                            frame: pane.frame().clone(),
                                        }
                                        .save("df.msv.bin", Format::Ron)
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
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn Storage) {
        set_value(storage, APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.panels(ctx);
        self.drag_and_drop(ctx);
        if self.reactive {
            ctx.request_repaint();
        }
    }
}

// fn bin(dropped_file: &DroppedFile) -> Result<DataFrame> {
//     Ok(bincode::deserialize(&dropped_file.bytes()?)?)
// }

mod computers;
mod data;
mod panes;
mod states;
mod widgets;
