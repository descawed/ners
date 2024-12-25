use std::future::Future;
use std::io::Cursor;
use std::sync::{Arc, Mutex};

use eframe::Frame;
use egui::Context;
use log::{debug, error, info};
use rfd::AsyncFileDialog;

use crate::rom::Cartridge;

// TODO: add serialization for app
pub struct NersApp {
    cartridge: Arc<Mutex<Option<Cartridge>>>,
}

impl Default for NersApp {
    fn default() -> Self {
        Self {
            cartridge: Arc::new(Mutex::new(None)),
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
                        let cartridge = self.cartridge.clone();
                        Self::spawn_async(async move {
                            let file = AsyncFileDialog::new()
                                .add_filter("NES ROMs", &["nes"/*, "unf", "unif"*/])
                                .pick_file()
                                .await;
                            
                            let Some(file) = file else {
                                debug!("User cancelled ROM selection");
                                return;
                            };

                            info!("User selected ROM at {}", file.path().as_os_str().to_str().unwrap_or("<unknown>"));
                            let data = file.read().await;
                            
                            match (cartridge.lock(), Cartridge::from_rom(Cursor::new(data))) {
                                (Ok(mut cartridge_ref), Ok(cartridge)) => *cartridge_ref = Some(cartridge),
                                (Err(e), _) => error!("Failed to lock ROM path for update: {:?}", e),
                                (_, Err(e)) => error!("Failed to load ROM: {}", e),
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