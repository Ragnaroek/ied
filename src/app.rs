use eframe::egui;
use egui::{CentralPanel, Color32, Frame, Pos2, Rect, RichText, ScrollArea, Stroke, vec2};

pub struct IEd {
    menu_expanded: bool,
    selected_cell: Option<(usize, usize)>,
}

impl eframe::App for IEd {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top panel for the burger menu button
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Burger menu button
                if ui.button("☰").clicked() {
                    self.menu_expanded = !self.menu_expanded;
                }
                ui.label("IED");
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

        egui::SidePanel::right("detail_panel")
            .min_width(250.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    if let Some((x, y)) = self.selected_cell {
                        ui.label(format!("x: {}, y: {}", x, y));
                    }
                });
            });

        CentralPanel::default().show(ctx, |ui| {
            let spacing = &mut ui.style_mut().spacing;
            spacing.item_spacing = egui::vec2(0.0, 0.0);
            spacing.button_padding = egui::vec2(0.0, 0.0);
            ScrollArea::both().show(ui, |ui| {
                let cell_rect = ui.available_width() / 64.0;
                for row in 0..64 {
                    ui.horizontal(|ui| {
                        for col in 0..64 {
                            let (rect, response) =
                                ui.allocate_exact_size(vec2(20.0, 20.0), egui::Sense::click());

                            if response.clicked() {
                                self.selected_cell = Some((row, col));
                            }

                            if self.selected_cell == Some((row, col)) {
                                ui.painter()
                                    .rect_filled(rect, 0.0, egui::Color32::LIGHT_BLUE);
                            }

                            ui.painter().rect_stroke(
                                rect,
                                0.0,
                                egui::Stroke::new(0.5, egui::Color32::GRAY),
                                egui::StrokeKind::Outside,
                            );
                        }
                    });
                }
            });
            ui.label("ied");
        });
    }
}

impl IEd {
    pub fn new(cc: &eframe::CreationContext<'_>) -> IEd {
        IEd {
            menu_expanded: true,
            selected_cell: None,
        }
    }
}
