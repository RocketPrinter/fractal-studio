#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use eframe::wgpu;

mod app;
mod fractal;
mod wgsl;
mod math;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let mut native_options = eframe::NativeOptions::default();
    native_options.wgpu_options.device_descriptor = device_descriptor();
    eframe::run_native(
        "Fractal Visualizer",
        native_options,
        Box::new(|cc| Box::new(app::EguiApp::new(cc))),
    )
}

// When compiling to web using trunk:
// todo: Caching is annoying and often requires Ctrl+F5
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Info).ok();

    let mut web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::new(app::EguiApp::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}

// copy pasted from default and modified
fn device_descriptor() -> Arc<dyn Fn(&wgpu::Adapter) -> wgpu::DeviceDescriptor<'static>> {
    Arc::new(|_adapter| {
        wgpu::DeviceDescriptor {
            label: Some("egui wgpu device"),
            features: wgpu::Features::default(),
            limits: wgpu::Limits::downlevel_webgl2_defaults(),
        }
    })
}