use eframe::Frame;
use egui::Context;
use log::info;

// TODO: add serialization for app
pub struct NersApp {

}

impl Default for NersApp {
    fn default() -> Self {
        Self { }
    }
}

impl NersApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // TODO: persistence
        Default::default()
    }
}

impl eframe::App for NersApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        info!("User clicked File > Open");
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let texture = ctx.load_texture("display", egui::ColorImage::example(), Default::default());
            ui.image(egui::load::SizedTexture::new(texture.id(), texture.size_vec2()));
        });
    }
}