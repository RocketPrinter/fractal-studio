#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod fractal;
mod wgsl;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    // use std::sync::Arc;
    // use eframe::egui_wgpu::WgpuSetup;
    // use eframe::wgpu;

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions::default();
    // let WgpuSetup::CreateNew(ref mut setup) = native_options.wgpu_options.wgpu_setup else { unreachable!(); };
    // setup.device_descriptor = Arc::new(|_adapter|
    //     wgpu::DeviceDescriptor {
    //         label: Some("egui wgpu device"),
    //         required_features: wgpu::Features::default(),
    //         // since the web version is limited to webgl2, it's helpful to limit the features so I can test the native version
    //         required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
    //         memory_hints: Default::default(),
    //     }
    // );

    eframe::run_native(
        "Fractal Visualizer",
        native_options,
        Box::new(|cc| Ok(Box::new(app::EguiApp::new(cc)))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

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
                Box::new(|cc| Ok(Box::new(app::EguiApp::new(cc)))),
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
}
