use egui::{CentralPanel, ColorImage, Pos2, Rect, TextureHandle, Vec2};
use iw::{
    gamedata::{GamedataHeaders, TextureData},
    map::{MapFileType, MapSegs, MapType},
};
use std::collections::HashMap;
use std::io::Cursor;

use crate::app::EditorWidget;

const TEXTURE_WIDTH: usize = 64;
const TEXTURE_HEIGHT: usize = 64;

pub struct WolfUpload {
    pub map_file: Vec<u8>,
    pub map_header_file: Vec<u8>,
    pub game_data_file: Vec<u8>,
}

struct WolfData {
    offsets: MapFileType,
    map_headers: Vec<MapType>,
    map_data: Vec<u8>,
    game_data: Vec<u8>,
    game_data_headers: GamedataHeaders,
}

#[derive(Copy, Clone)]
struct Tile {
    x: usize,
    y: usize,
    wall: u16,
    info: u16,
}

pub struct WolfEditor {
    data: WolfData,
    map: MapSegs,
    menu_expanded: bool,
    selected_tile: Option<Tile>,

    textures: HashMap<String, TextureHandle>,
}

impl WolfEditor {
    pub fn new(files: WolfUpload) -> Result<WolfEditor, String> {
        let offsets = iw::map::load_map_offsets(&files.map_header_file)?;
        let (offsets, map_headers) = iw::map::load_map_headers(&files.map_file, offsets)?;
        let mut cursor = Cursor::new(&files.map_file);
        let map = iw::map::load_map(&mut cursor, &map_headers, &offsets, 0)?;

        let game_data_headers = iw::gamedata::load_gamedata_headers(&files.game_data_file)?;

        Ok(WolfEditor {
            data: WolfData {
                offsets,
                map_headers,
                map_data: files.map_file,
                game_data: files.game_data_file,
                game_data_headers,
            },
            map,
            menu_expanded: true,
            selected_tile: None,
            textures: HashMap::new(),
        })
    }

    fn texture_image(
        &mut self,
        ui: &mut egui::Ui,
        key: String,
        texture: &TextureData,
    ) -> TextureHandle {
        if let Some(handle) = self.textures.get(&key) {
            log::debug!("using cached texture!");
            return handle.clone();
        }

        let mut image_data = vec![0; TEXTURE_WIDTH * TEXTURE_HEIGHT * 3];
        for x in 0..TEXTURE_WIDTH {
            for y in 0..TEXTURE_HEIGHT {
                let ix = TEXTURE_WIDTH * y + x;
                let colour = iw::assets::gamepal_color(texture.bytes[ix] as usize);
                image_data[ix * 3 + 0] = colour.r;
                image_data[ix * 3 + 1] = colour.g;
                image_data[ix * 3 + 2] = colour.b;
            }
        }

        let image = ColorImage::from_rgb([TEXTURE_WIDTH, TEXTURE_HEIGHT], &image_data);
        let handle = ui.ctx().load_texture(&key, image, Default::default());
        self.textures.insert(key, handle.clone());
        handle
    }

    fn render_texture(&mut self, ui: &mut egui::Ui, which: u16, dir: bool) {
        let header = &self.data.game_data_headers.headers
            [(which as usize - 1) * 2 + if dir { 1 } else { 0 }];
        if header.length == 4096 {
            let texture =
                iw::gamedata::load_texture(&mut Cursor::new(&self.data.game_data), header)
                    .expect("texture");
            let img = self.texture_image(
                ui,
                format!("{}-wall{}", if dir { "v" } else { "h" }, which),
                &texture,
            );
            if dir {
                ui.label("vertical:");
            } else {
                ui.label("horizontal:");
            }
            ui.image(&img);
        }
    }
}

impl EditorWidget for WolfEditor {
    fn show(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("☰").clicked() {
                    self.menu_expanded = !self.menu_expanded;
                }
                ui.label("Wolfenstein 3-D");
            });
        });

        let menu_width = if self.menu_expanded { 200.0 } else { 40.0 };
        egui::SidePanel::left("menu_panel")
            .resizable(false)
            .width_range(menu_width..=menu_width)
            .show(ctx, |ui| {
                if self.menu_expanded {
                    ui.vertical(|ui| {
                        ui.label("Editor");
                        ui.label("Graphics");
                        ui.label("Texture/Sprites");
                    });
                } else {
                    // Show only icons or minimal UI when collapsed
                    ui.vertical_centered(|ui| {
                        ui.label("TODO");
                    });
                }
            });

        let mut editor_rect = Rect::from_pos(Pos2::new(0.0, 0.0));
        CentralPanel::default().show(ctx, |ui| {
            let panel_rect = ui.max_rect();
            let cell_dim = panel_rect.width().min(panel_rect.height()) / 64.0;
            editor_rect = Rect::from_min_max(
                panel_rect.min,
                Pos2::new(
                    panel_rect.min.x + cell_dim * 64.0,
                    panel_rect.min.y + cell_dim * 64.0,
                ),
            );
            for x in 0..64 {
                for y in 0..64 {
                    let ptr = y * 64 + x;
                    let wall = self.map.segs[0][ptr];
                    let info = self.map.segs[1][ptr];

                    let tile = Tile { x, y, wall, info };

                    let rect = Rect::from_min_size(
                        Pos2::new(
                            panel_rect.min.x + x as f32 * cell_dim,
                            panel_rect.min.y + y as f32 * cell_dim,
                        ),
                        Vec2::new(cell_dim, cell_dim),
                    );
                    let response = ui.interact(
                        rect,
                        egui::Id::new(format!("({},{}", x, y)),
                        egui::Sense::click(),
                    );
                    if response.clicked() {
                        self.selected_tile = Some(tile);
                    }

                    render_wall(ui, rect, &tile);

                    if let Some(tile) = &self.selected_tile {
                        if tile.x == x && tile.y == y {
                            ui.painter()
                                .rect_filled(rect, 0.0, egui::Color32::LIGHT_BLUE);
                        }
                    }
                }
            }
        });

        egui::Area::new("tile_editor".into())
            .movable(false)
            .order(egui::Order::Foreground)
            .current_pos(egui::pos2(editor_rect.max.x + 20.0, editor_rect.min.y))
            .show(ctx, |ui| {
                let painter = ui.painter();
                let rect = ui.max_rect();
                painter.rect_filled(rect, 0.0, ui.style().visuals.panel_fill);
                painter.rect_stroke(
                    rect,
                    0.0,
                    egui::Stroke::new(
                        1.0,
                        ui.style().visuals.widgets.noninteractive.bg_stroke.color,
                    ),
                    egui::StrokeKind::Outside,
                );

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.add_space(5.0);
                    ui.label(
                        egui::RichText::new("Tile Editor")
                            .strong()
                            .color(ui.style().visuals.strong_text_color()),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        ui.add_space(5.0);
                    });
                });
                ui.separator();

                let wall = if let Some(selected) = &self.selected_tile {
                    ui.label(format!("x: {}, y: {}", selected.x, selected.y));
                    ui.label(format!("Wall Tile: {}", selected.wall));
                    ui.label(format!("Info Tile: {}", selected.info));
                    Some(selected.wall)
                } else {
                    ui.label("Nothing selected");
                    None
                };

                if let Some(wall) = wall {
                    ui.add_space(10.0);
                    self.render_texture(ui, wall, true);
                    ui.add_space(10.0);
                    self.render_texture(ui, wall, false);
                }

                ui.add_space(30.0);
            });
    }
}

fn render_wall(ui: &mut egui::Ui, rect: Rect, tile: &Tile) {
    if tile.wall < 107 {
        ui.painter().rect_filled(rect, 0.0, egui::Color32::GRAY);
    } else {
        ui.painter().rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(0.5, egui::Color32::GRAY),
            egui::StrokeKind::Outside,
        );
    }
}
