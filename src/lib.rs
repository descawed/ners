use std::fs::File;

use eframe::egui;

mod app;
mod rom;
mod hw;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
pub fn launch() -> eframe::Result {
    use simplelog::*;

    let _ = CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Info, Config::default(), File::create("ners.log").map_err(|e| eframe::Error::AppCreation(Box::new(e)))?),
        ],
    );

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([341.0, 302.0]) // +40 pixels height for menu
            .with_min_inner_size([341.0, 262.0]),
        ..Default::default()
    };

    let rt = tokio::runtime::Runtime::new().map_err(|e| eframe::Error::AppCreation(Box::new(e)))?;

    rt.block_on(async {
        eframe::run_native(
            "ners",
            native_options,
            Box::new(|cc| Ok(Box::new(app::NersApp::new(cc)))),
        )
    })?;

    // don't wait for the console thread if it's still running
    rt.shutdown_background();

    Ok(())
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
pub fn launch() -> eframe::Result {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).map_err(|e| eframe::Error::AppCreation(Box::new(e)))?;

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(app::NersApp::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });

    Ok(())
}