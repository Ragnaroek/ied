use egui::{
    CentralPanel, Color32, ColorImage, ComboBox, Image, Pos2, Rect, TextureHandle, TextureId, Vec2,
};
use iw::{
    gamedata::{GamedataHeaders, TextureData},
    map::{MapFileType, MapSegs, MapType},
    web::load_map,
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
    map_data: MapSegs,
    menu_expanded: bool,
    selected_tile: Option<Tile>,

    textures: HashMap<String, TextureHandle>,

    episode: usize,
    map: usize,
}

enum Dir {
    North,
    South,
    West,
    East,
}

impl Dir {
    fn from_num(n: u16) -> Dir {
        match n {
            0 => Dir::North,
            1 => Dir::East,
            2 => Dir::South,
            3 => Dir::West,
            _ => Dir::North,
        }
    }

    /// in radian
    fn text_rotation(&self) -> f32 {
        // text naturally faces to the east already
        let degree = match self {
            Dir::North => 270.0,
            Dir::East => 0.0,
            Dir::South => 90.0,
            Dir::West => 180.0,
        };
        degree * (std::f32::consts::PI / 180.0)
    }
}

impl WolfEditor {
    pub fn new(files: WolfUpload) -> Result<WolfEditor, String> {
        let offsets = iw::map::load_map_offsets(&files.map_header_file)?;
        let (offsets, map_headers) = iw::map::load_map_headers(&files.map_file, offsets)?;
        let mut cursor = Cursor::new(&files.map_file);
        let map_data = iw::map::load_map(&mut cursor, &map_headers, &offsets, 0)?;

        let game_data_headers = iw::gamedata::load_gamedata_headers(&files.game_data_file)?;

        Ok(WolfEditor {
            data: WolfData {
                offsets,
                map_headers,
                map_data: files.map_file,
                game_data: files.game_data_file,
                game_data_headers,
            },
            map_data,
            menu_expanded: true,
            selected_tile: None,
            textures: HashMap::new(),
            episode: 0,
            map: 0,
        })
    }

    fn reload_map(&mut self) -> Result<(), String> {
        let mut cursor = Cursor::new(&self.data.map_data);
        let map_data = iw::map::load_map(
            &mut cursor,
            &self.data.map_headers,
            &self.data.offsets,
            self.episode * 10 + self.map,
        )?;
        self.map_data = map_data;
        Ok(())
    }

    fn tex_wall(&mut self, ui: &mut egui::Ui, key: String, texture: &TextureData) -> TextureHandle {
        if let Some(handle) = self.textures.get(&key) {
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

    fn tex_checker(&mut self, ui: &mut egui::Ui) -> TextureHandle {
        let key = "tex-checker";
        if let Some(handle) = self.textures.get(key) {
            return handle.clone();
        }

        let mut checker_data = vec![0u8; 8 * 8 * 4];
        for y in 0..8 {
            for x in 0..8 {
                let i = (y * 8 + x) * 4;
                let (color, alpha) = if (x / 2 + y / 2) % 2 == 0 {
                    (0xFF, 0x00)
                } else {
                    (0x00, 0xFF)
                };
                checker_data[i] = color; // R
                checker_data[i + 1] = color; // G
                checker_data[i + 2] = color; // B
                checker_data[i + 3] = alpha; // A
            }
        }

        let handle = ui.ctx().load_texture(
            key,
            ColorImage::from_rgba_unmultiplied([8, 8], &checker_data),
            Default::default(),
        );
        self.textures.insert(key.to_string(), handle.clone());
        handle
    }

    fn tex_door(&mut self, ui: &mut egui::Ui, vert: bool) -> TextureHandle {
        let key = if vert { "tex-door-v" } else { "tex-door-h" };
        if let Some(handle) = self.textures.get(key) {
            return handle.clone();
        }

        let mut tex_data = vec![0u8; 8 * 8 * 4];

        let x_range = if vert { 2..5 } else { 0..8 };

        for x in x_range {
            let y_range = if vert { 0..8 } else { 2..5 };
            for y in y_range {
                let i = (y * 8 + x) * 4;
                tex_data[i] = 0x84;
                tex_data[i + 1] = 0x84;
                tex_data[i + 2] = 0x84;
                tex_data[i + 3] = 0xFF;
            }
        }

        let handle = ui.ctx().load_texture(
            key,
            ColorImage::from_rgba_unmultiplied([8, 8], &tex_data),
            Default::default(),
        );
        self.textures.insert(key.to_string(), handle.clone());
        handle
    }

    fn render_tile(&mut self, ui: &mut egui::Ui, rect: Rect, tile: &Tile) {
        self.render_tile_background(ui, rect, tile.wall);

        // grid
        ui.painter().rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.0, GRID_COLOUR),
            egui::StrokeKind::Outside,
        );

        // info layer

        let (info_icon, dir) = match tile.info {
            19 | 20 | 21 | 22 => (
                egui_phosphor::regular::ARROW_RIGHT,
                Dir::from_num(tile.info - 19),
            ),
            _ => ("", Dir::North),
        };

        if !info_icon.is_empty() {
            let galley = ui.painter().layout_no_wrap(
                info_icon.to_string(),
                egui::FontId::default(),
                Color32::WHITE,
            );
            let (dis_x, dis_y) = match dir {
                Dir::East => (0.0, 0.0),
                Dir::South => (galley.size().x, 0.0),
                Dir::West => (galley.size().x, galley.size().y),
                Dir::North => (0.0, galley.size().y),
            };
            let x_pad = (rect.width() - galley.size().x) / 2.0;
            let y_pad = (rect.height() - galley.size().y) / 2.0;

            let text_shape = egui::Shape::Text(egui::epaint::TextShape {
                pos: Pos2 {
                    x: rect.min.x + dis_x + x_pad,
                    y: rect.min.y + dis_y + y_pad,
                },
                galley,
                underline: egui::Stroke::NONE,
                override_text_color: None,
                fallback_color: Color32::WHITE,
                opacity_factor: 1.0,
                angle: dir.text_rotation(),
            });

            ui.painter().add(text_shape);
        }
    }

    fn render_tile_background(&mut self, ui: &mut egui::Ui, rect: Rect, wall: u16) {
        if wall < 107 {
            match wall {
                1 => self.bg_colour(ui, rect, Color32::from_rgb(0x84, 0x84, 0x84)),
                2 => self.bg_checker(ui, rect, Color32::from_rgb(0x84, 0x84, 0x84)),
                8 => self.bg_colour(ui, rect, Color32::from_rgb(0x00, 0x00, 0x84)),
                9 => self.bg_checker(ui, rect, Color32::from_rgb(0x00, 0x00, 0x84)),
                12 => self.bg_colour(ui, rect, Color32::from_rgb(0xA5, 0x00, 0x00)),
                90 => self.bg_door(ui, rect, true),
                91 => self.bg_door(ui, rect, false),
                _ => self.bg_colour(ui, rect, Color32::from_rgb(0x84, 0x84, 0x84)),
            }
        } else {
            self.bg_colour(ui, rect, Color32::BLACK);
        };
    }

    fn bg_checker(&mut self, ui: &mut egui::Ui, rect: Rect, bg_colour: Color32) {
        let tex_id = self.tex_checker(ui).id();
        self.bg_colour(ui, rect, bg_colour);
        self.bg_tex(ui, rect, tex_id);
    }

    fn bg_colour(&self, ui: &mut egui::Ui, rect: Rect, colour: Color32) {
        ui.painter().rect_filled(rect, 0.0, colour);
    }

    fn bg_door(&mut self, ui: &mut egui::Ui, rect: Rect, vert: bool) {
        let tex_id = self.tex_door(ui, vert).id();
        self.bg_colour(ui, rect, Color32::BLACK);
        self.bg_tex(ui, rect, tex_id);
    }

    fn bg_tex(&self, ui: &mut egui::Ui, rect: Rect, handle: TextureId) {
        ui.painter().image(
            handle,
            rect,
            Rect::from_min_max(Pos2 { x: 0.0, y: 0.0 }, Pos2 { x: 1.0, y: 1.0 }),
            Color32::WHITE,
        );
    }

    fn render_texture(&mut self, ui: &mut egui::Ui, which: u16, dir: bool) {
        let header = &self.data.game_data_headers.headers
            [(which as usize - 1) * 2 + if dir { 1 } else { 0 }];
        if header.length == 4096 {
            let texture =
                iw::gamedata::load_texture(&mut Cursor::new(&self.data.game_data), header)
                    .expect("texture");
            let img = self.tex_wall(
                ui,
                format!("{}-wall{}", if dir { "v" } else { "h" }, which),
                &texture,
            );
            if dir {
                ui.label("vertical:");
            } else {
                ui.label("horizontal:");
            }

            let rotated_image =
                Image::new(&img).rotate(std::f32::consts::PI / 2.0, egui::Vec2::splat(0.5));
            ui.add(rotated_image);
        }
    }

    fn handle_keys(&mut self, ctx: &egui::Context) {
        let mut dis = None;
        if ctx.input(|i| i.key_down(egui::Key::ArrowUp)) {
            dis = Some((0, -1));
        }
        if ctx.input(|i| i.key_down(egui::Key::ArrowDown)) {
            dis = Some((0, 1));
        }
        if ctx.input(|i| i.key_down(egui::Key::ArrowLeft)) {
            dis = Some((-1, 0));
        }
        if ctx.input(|i| i.key_down(egui::Key::ArrowRight)) {
            dis = Some((1, 0));
        }

        if let Some(dis) = dis
            && let Some(selected) = self.selected_tile
        {
            let x = (selected.x as isize + dis.0).clamp(0, 63) as usize;
            let y = (selected.y as isize + dis.1).clamp(0, 63) as usize;
            self.selected_tile = Some(self.tile_at(x, y))
        }
    }

    fn tile_at(&self, x: usize, y: usize) -> Tile {
        let ptr = y * 64 + x;
        let wall = self.map_data.segs[0][ptr];
        let info = self.map_data.segs[1][ptr];
        Tile { x, y, wall, info }
    }
}

impl EditorWidget for WolfEditor {
    fn show(&mut self, ctx: &egui::Context) {
        self.handle_keys(ctx);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("☰").clicked() {
                    self.menu_expanded = !self.menu_expanded;
                }
                ui.label("Wolfenstein 3-D");
                ui.add_space(70.0);

                let episode_before = self.episode;
                ui.label("Episode");
                ComboBox::from_id_salt("cb_episode")
                    .width(60.0)
                    .selected_text(format!("{}", self.episode + 1))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.episode, 0, "1");
                        ui.selectable_value(&mut self.episode, 1, "2");
                        ui.selectable_value(&mut self.episode, 2, "3");
                        ui.selectable_value(&mut self.episode, 3, "4");
                        ui.selectable_value(&mut self.episode, 4, "5");
                        ui.selectable_value(&mut self.episode, 5, "6");
                    });
                if episode_before != self.episode {
                    self.reload_map().expect("map reload");
                }

                ui.add_space(15.0);

                let map_before = self.map;
                ui.label("Map");
                ComboBox::from_id_salt("cb_map")
                    .width(60.0)
                    .selected_text(format!("{}", self.map + 1))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.map, 0, "1");
                        ui.selectable_value(&mut self.map, 1, "2");
                        ui.selectable_value(&mut self.map, 2, "3");
                        ui.selectable_value(&mut self.map, 3, "4");
                        ui.selectable_value(&mut self.map, 4, "5");
                        ui.selectable_value(&mut self.map, 5, "6");
                        ui.selectable_value(&mut self.map, 6, "7");
                        ui.selectable_value(&mut self.map, 7, "8");
                        ui.selectable_value(&mut self.map, 8, "9");
                        ui.selectable_value(&mut self.map, 9, "10");
                    });
                if map_before != self.map {
                    self.reload_map().expect("map reload");
                }
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
                    let tile = self.tile_at(x, y);

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

                    self.render_tile(ui, rect, &tile);

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

const GRID_COLOUR: Color32 = Color32::from_rgb(0x55, 0x55, 0x55);
