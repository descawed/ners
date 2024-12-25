use std::future::Future;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use eframe::Frame;
use egui::Context;
use log::{debug, error, info};
use rfd::AsyncFileDialog;

// TODO: add serialization for app
pub struct NersApp {
    rom_path: Arc<Mutex<Option<PathBuf>>>,
}

impl Default for NersApp {
    fn default() -> Self {
        Self {
            rom_path: Arc::new(Mutex::new(None)),
        }
    }
}

impl NersApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // TODO: persistence
        Default::default()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn spawn_async<F: Future<Output = ()> + Send + 'static>(future: F) {
        tokio::task::spawn(future);
    }

    #[cfg(target_arch = "wasm32")]
    fn spawn_async<F: Future<Output = ()> + Send + 'static>(future: F) {
        wasm_bindgen_futures::spawn_local(future);
    }
}

impl eframe::App for NersApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        let rom_path = self.rom_path.clone();
                        Self::spawn_async(async move {
                            let file = AsyncFileDialog::new()
                                .add_filter("NES ROMs", &["nes", "unf", "unif"])
                                .pick_file()
                                .await;
                            
                            match (rom_path.lock(), file.map(|h| h.path().to_path_buf())) {
                                (Ok(mut rom_path_ref), Some(path)) => {
                                    info!("User selected ROM at {}", path.as_os_str().to_str().unwrap_or("<unknown>"));
                                    *rom_path_ref = Some(path);
                                }
                                (Err(e), _) => error!("Failed to lock ROM path for update: {:?}", e),
                                (_, None) => debug!("User cancelled ROM selection"),
                            }
                        });
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