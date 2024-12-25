use std::future::Future;
use std::io::Cursor;
use std::sync::{Arc, Condvar, Mutex};

use eframe::Frame;
use egui::Context;
use log::{debug, error, info};
use rfd::AsyncFileDialog;

use crate::hw::Nes;
use crate::rom::Cartridge;

// TODO: add serialization for app
pub struct NersApp {
    console: Arc<Mutex<Nes>>,
    frame_signal: Arc<Condvar>,
}

impl Default for NersApp {
    fn default() -> Self {
        Self {
            console: Arc::new(Mutex::new(Nes::new())),
            frame_signal: Arc::new(Condvar::new()),
        }
    }
}

impl NersApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // TODO: persistence
        let app = Self::default();
        // start the console execution task
        Self::spawn_async(Self::run_game(Arc::clone(&app.console), Arc::clone(&app.frame_signal)));
        app
    }
    
    async fn run_game(console: Arc<Mutex<Nes>>, frame_signal: Arc<Condvar>) {
        loop {
            // wait until we receive the frame signal while the console is running
            let mut console = frame_signal.wait(console.lock().unwrap()).unwrap();
            while !console.is_running() {
                console = frame_signal.wait(console).unwrap();
            }
            
            if let Err(e) = console.run() {
                error!("Console execution failed: {}", e);
                console.pause();
            }
        }
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
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        let console = self.console.clone();
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
                            
                            match (console.lock(), Cartridge::from_rom(Cursor::new(data))) {
                                (Ok(mut console_ref), Ok(cartridge)) => console_ref.load_cartridge(cartridge),
                                (Err(e), _) => error!("Failed to lock ROM path for update: {}", e),
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

        // run the active game, if any
        let console = self.console.lock().unwrap();
        if console.is_running() {
            // while a game is running, redraw the UI regularly
            ctx.request_repaint();
            // signal the console thread to run for another frame
            self.frame_signal.notify_one();
        }
    }
}