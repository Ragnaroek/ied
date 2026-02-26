use eframe::egui;

use crate::wolf::WolfEditor;

pub trait EditorWidget {
    fn show(&mut self, ctx: &egui::Context);
}

pub struct IEd {
    editor: Option<Box<dyn EditorWidget>>,
}

const NUM_START_TILES: usize = 2;
const TILE_DIMENSION: f32 = 200.0;

impl eframe::App for IEd {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(editor) = &mut self.editor {
            editor.show(ctx);
        } else {
            // placeholder startpage, will be made nicer
            egui::CentralPanel::default().show(ctx, |ui| {
                let width = ui.available_width();
                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        ui.horizontal(|ui| {
                            ui.add_space((width - (NUM_START_TILES as f32 * TILE_DIMENSION)) / 2.0);

                            // Allocate space for the tiles
                            let tile_size = egui::vec2(200.0, 200.0);

                            // Wolfenstein 3D Tile
                            egui::Frame::dark_canvas(ui.style())
                                .stroke(egui::Stroke::new(
                                    2.0,
                                    egui::Color32::from_rgb(0, 128, 128),
                                ))
                                .show(ui, |ui| {
                                    ui.set_width(tile_size.x);
                                    ui.set_height(tile_size.y);
                                    ui.vertical_centered(|ui| {
                                        ui.label(
                                            egui::RichText::new("Wolfenstein 3D")
                                                .color(egui::Color32::WHITE)
                                                .size(16.0),
                                        );
                                        ui.separator();
                                        if ui.button("Create").clicked() {
                                            println!("Create Wolfenstein 3D Map");
                                        }
                                        if ui.button("Edit").clicked() {
                                            println!("Edit Wolfenstein 3D Map");
                                        }
                                        ui.add_space(25.0);
                                        ui.label(
                                            "You need to upload the following files for edit:",
                                        );
                                        ui.label("GAMEMAPS.WLX");
                                        ui.label("MAPHEAD.WLX");
                                        ui.label("VSWAP.WLX");
                                    });
                                });

                            // Doom Tile
                            egui::Frame::dark_canvas(ui.style())
                                .stroke(egui::Stroke::new(
                                    2.0,
                                    egui::Color32::from_rgb(255, 60, 60),
                                ))
                                .show(ui, |ui| {
                                    ui.set_width(tile_size.x);
                                    ui.set_height(tile_size.y);
                                    ui.vertical_centered(|ui| {
                                        ui.label(
                                            egui::RichText::new("Doom")
                                                .color(egui::Color32::WHITE)
                                                .size(16.0),
                                        );
                                        ui.separator();
                                        if ui.button("Create").clicked() {
                                            println!("Create Doom Map");
                                        }
                                        if ui.button("Edit").clicked() {
                                            println!("Edit Doom Map");
                                        }
                                    });
                                });
                        });
                    },
                );
            });
        }
    }
}

impl IEd {
    pub fn new(cc: &eframe::CreationContext<'_>) -> IEd {
        IEd { editor: None }
    }
}
