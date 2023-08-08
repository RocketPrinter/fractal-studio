#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use eframe::wgpu;

mod app;
mod fractal;
mod wgsl;

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
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let mut web_options = eframe::WebOptions::default();
    // disallow WebGPU as it doesn't support Push Constants
    //web_options.wgpu_options.supported_backends &= !eframe::wgpu::Backends::BROWSER_WEBGPU;
    //web_options.wgpu_options.device_descriptor = device_descriptor();
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
        /*let base_limits = if adapter.get_info().backend == wgpu::Backend::Gl {
            wgpu::Limits::downlevel_webgl2_defaults()
        } else {
            wgpu::Limits::default()
        };*/
        let base_limits = wgpu::Limits::downlevel_webgl2_defaults();
        wgpu::DeviceDescriptor {
            label: Some("egui wgpu device"),
            features: wgpu::Features::default() | wgpu::Features::PUSH_CONSTANTS,
            limits: base_limits,
        }
    })
}