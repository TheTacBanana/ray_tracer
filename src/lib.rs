use context::GraphicsContext;
use window::Window;
use winit::{event::{Event, WindowEvent}, event_loop::ControlFlow};
use anyhow::Result;
use cfg_if::cfg_if;

pub mod context;
pub mod window;
pub mod vertex;
pub mod thread_context;
pub mod pipeline;
pub mod camera;

/// Load bytes from path, if compiled for web then do via http request
pub async fn load_bytes(path: &str) -> Result<Vec<u8>> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let window = web_sys::window().unwrap();
            let origin = window.origin();
            let base = reqwest::Url::parse(&format!("{}/", origin,)).unwrap();
            let path = base.join(path).unwrap();
            let bytes = reqwest::get(path)
                .await?
                .bytes()
                .await?;
        } else {
            let bytes = std::fs::read(path)?;
        }
    }
    Ok(bytes.to_vec())
}


/// Entry point for web
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    // Create a window and graphics context
    let window = Window::new();
    let mut context = GraphicsContext::new(&window).await;

    window.run(move |window, event, control_flow| {
        // Handle Winit Events
        match event {
            // Render everything
            Event::RedrawRequested(_) => {
                context.render(window).unwrap();
            }
            // Trigger a resize
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                context.resize(size.width, size.height);
            }
            // Redraw if requested
            Event::MainEventsCleared | Event::UserEvent(_) => {
                window.request_redraw();
            }
            // Exit if close request
            Event::LoopDestroyed
            | Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
