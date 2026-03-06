use egui::{CentralPanel, Pos2, Rect, Vec2};
use iw::map::{MapFileType, MapSegs, MapType};
use std::io::Cursor;

use crate::app::EditorWidget;

pub struct WolfUpload {
    pub map_file: Vec<u8>,
    pub header_file: Vec<u8>,
    pub game_data_file: Vec<u8>,
}

struct WolfFiles {
    offsets: MapFileType,
    headers: Vec<MapType>,
    map_data: Vec<u8>,
}

#[derive(Copy, Clone)]
struct Tile {
    x: usize,
    y: usize,
    wall: u16,
}

pub struct WolfEditor {
    files: WolfFiles,
    map: MapSegs,
    menu_expanded: bool,
    selected_tile: Option<Tile>,
}

impl WolfEditor {
    pub fn new(files: WolfUpload) -> Result<WolfEditor, String> {
        let offsets = iw::map::load_map_offsets(&files.header_file)?;
        let (offsets, headers) = iw::map::load_map_headers(&files.map_file, offsets)?;
        let mut cursor = Cursor::new(&files.map_file);
        let map = iw::map::load_map(&mut cursor, &headers, &offsets, 0)?;

        Ok(WolfEditor {
            files: WolfFiles {
                offsets,
                headers,
                map_data: files.map_file,
            },
            map,
            menu_expanded: true,
            selected_tile: None,
        })
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

                    let tile = Tile { x, y, wall };

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

                if let Some(selected) = &self.selected_tile {
                    ui.label(format!("x: {}, y: {}", selected.x, selected.y));
                    ui.label(format!("Wall Tile: {}", selected.wall));
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
