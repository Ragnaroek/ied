use std::cell;

use egui::{CentralPanel, Pos2, Rect, ScrollArea, Vec2, vec2};

use crate::app::EditorWidget;

pub struct WolfFiles {
    pub map_file: Vec<u8>,
    pub header_file: Vec<u8>,
    pub game_data_file: Vec<u8>,
}

pub struct WolfEditor {
    files: WolfFiles,
    menu_expanded: bool,
    selected_cell: Option<(usize, usize)>,
}

impl WolfEditor {
    pub fn new(files: WolfFiles) -> WolfEditor {
        WolfEditor {
            files,
            menu_expanded: true,
            selected_cell: None,
        }
    }
}

impl EditorWidget for WolfEditor {
    fn show(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Burger menu button
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
            for row in 0..64 {
                for col in 0..64 {
                    let rect = Rect::from_min_size(
                        Pos2::new(
                            panel_rect.min.x + row as f32 * cell_dim,
                            panel_rect.min.y + col as f32 * cell_dim,
                        ),
                        Vec2::new(cell_dim, cell_dim),
                    );
                    let response = ui.interact(
                        rect,
                        egui::Id::new(format!("({},{}", col, row)),
                        egui::Sense::click(),
                    );
                    if response.clicked() {
                        self.selected_cell = Some((row, col));
                    }

                    if self.selected_cell == Some((row, col)) {
                        ui.painter()
                            .rect_filled(rect, 0.0, egui::Color32::LIGHT_BLUE);
                    } else {
                        ui.painter().rect_stroke(
                            rect,
                            0.0,
                            egui::Stroke::new(0.5, egui::Color32::GRAY),
                            egui::StrokeKind::Outside,
                        );
                    }
                }
            }
        });

        egui::Area::new("cell_editor".into())
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
                        egui::RichText::new("Cell Editor")
                            .strong()
                            .color(ui.style().visuals.strong_text_color()),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        ui.add_space(5.0);
                    });
                });
                ui.separator();

                if let Some(selected) = self.selected_cell {
                    ui.label(format!("x: {}, y: {}", selected.0, selected.1));
                }

                ui.add_space(30.0);
            });
    }
}
