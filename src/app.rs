use std::sync::Arc;

use eframe::egui;
use eframe::egui::{Button, Color32, ColorImage, Pos2, RichText, Vec2};
use egui::{Align2, FontDefinitions, FontFamily, FontId, Frame, Label, Rect, TextureHandle};
use poll_promise::Promise;

use crate::wolf::WolfEditor;

pub struct FileUpload {
    pub name: String,
    pub bytes: Vec<u8>,
}

pub trait EditorWidget {
    fn show(&mut self, ctx: &egui::Context);
}

pub struct IEd {
    editor: Option<Box<dyn EditorWidget>>,
    wolf_edit_file_promise: Option<Promise<Vec<FileUpload>>>,

    disk_image: TextureHandle,
}

const NUM_START_TILES: usize = 2;
const TILE_DIMENSION: f32 = 200.0;
const DISK_PADDING: f32 = 10.0;

const BUTTON_BACKGROUND: Color32 = Color32::from_rgb(30, 58, 80);
const BACKGROUND_COLOR: Color32 = egui::Color32::from_rgb(0x38, 0x4C, 0x71);

const FONT_NAME: &str = "press_start_2p";

impl eframe::App for IEd {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(editor) = &mut self.editor {
            editor.show(ctx);
        } else {
            egui::CentralPanel::default()
                .frame(Frame::NONE)
                .show(ctx, |ui| {
                    ui.painter()
                        .rect_filled(ui.max_rect(), 0.0, BACKGROUND_COLOR);

                    let w = ui.available_width();
                    let h = ui.available_height();
                    let w_disk = self.disk_image.size()[0] as f32;
                    let h_disk = self.disk_image.size()[1] as f32;
                    let y = h / 2.0 - h_disk / 2.0;

                    self.render_disk_tile(
                        ui,
                        Pos2::new(w / 2.0 - w_disk - DISK_PADDING, y),
                        Color32::from_rgb(0xE1, 0x41, 0x35),
                        "WOLFENSTEIN 3-D",
                    );
                    self.render_disk_tile(
                        ui,
                        Pos2::new(w / 2.0 + DISK_PADDING, y),
                        Color32::from_rgb(0x59, 0xBE, 0xB0),
                        "DOOM",
                    );
                });
        }
    }
}

impl IEd {
    pub fn new(cc: &eframe::CreationContext<'_>) -> IEd {
        let image_bytes = include_bytes!("../assets/floppy_disk.png");
        let image = image::load_from_memory(image_bytes).unwrap().to_rgba8();
        let image_size = [image.width() as usize, image.height() as usize];
        let pixels = image.into_raw();
        let color_image = ColorImage::from_rgba_unmultiplied(image_size, &pixels);
        let texture = cc
            .egui_ctx
            .load_texture("logo", color_image, egui::TextureOptions::LINEAR);

        setup_font(&cc.egui_ctx);

        IEd {
            editor: None,
            wolf_edit_file_promise: None,
            disk_image: texture,
        }
    }

    fn render_disk_tile(&self, ui: &mut egui::Ui, pos: Pos2, colour: Color32, text: &str) {
        let h = self.disk_image.size()[0] as f32;
        let w = self.disk_image.size()[1] as f32;
        let rect = Rect::from_min_size(pos, Vec2::new(w, h));
        ui.painter().image(
            self.disk_image.id(),
            rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );

        let overlay_rect =
            Rect::from_min_size(rect.min + Vec2::new(19.0, 49.0), Vec2::new(110.0, 45.0));
        ui.painter().rect_filled(overlay_rect, 5.0, colour);
        ui.put(
            Rect::from_min_max(
                overlay_rect.min + Vec2::new(5.0, 5.0),
                overlay_rect.max - Vec2::new(5.0, 5.0),
            ),
            Label::new(
                RichText::new(text)
                    .color(Color32::BLACK)
                    .font(egui::FontId::monospace(9.0)),
            ),
        );

        let button_size = Vec2::new(w / 2.0, 20.0);
        let button_pos = Pos2::new(pos.x + (w - button_size.x) / 2.0, pos.y + h + 20.0);
        ui.put(
            Rect::from_min_size(button_pos, button_size),
            Button::new(
                RichText::new("New")
                    .color(Color32::WHITE)
                    .strong()
                    .size(11.0)
                    .family(FontFamily::Monospace),
            )
            .fill(BUTTON_BACKGROUND)
            .corner_radius(4.0),
        );
        ui.put(
            Rect::from_min_size(
                Pos2::new(button_pos.x, button_pos.y + button_size.y + 10.0),
                button_size,
            ),
            Button::new(
                RichText::new("Edit")
                    .color(Color32::WHITE)
                    .strong()
                    .size(11.0)
                    .family(FontFamily::Monospace),
            )
            .fill(BUTTON_BACKGROUND)
            .corner_radius(4.0),
        );
    }

    fn start_edit_wolf(&mut self) {}
}

fn setup_font(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        FONT_NAME.to_owned(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../assets/PressStart2P-vaV7.ttf"
        ))),
    );

    fonts
        .families
        .insert(FontFamily::Monospace, vec![FONT_NAME.to_owned()]);

    ctx.set_fonts(fonts);
}

pub async fn open_file() -> FileUpload {
    let file = rfd::AsyncFileDialog::new().pick_file().await.unwrap();
    let bytes = file.read().await;
    FileUpload {
        name: file.file_name(),
        bytes,
    }
}
