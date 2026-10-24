use std::convert::Infallible;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::window::WindowBuilder;
use wry::WebViewBuilder;

use crate::blocker::{build_init_script, should_allow_navigation};
use crate::config::{APP_TITLE, SPOTIFY_URL, WINDOW_HEIGHT, WINDOW_WIDTH};

const ICON_PNG: &[u8] = include_bytes!("../../../assets/icon.png");

fn load_window_icon() -> Option<tao::window::Icon> {
    let mut reader = png::Decoder::new(ICON_PNG).read_info().ok()?;
    if reader.info().color_type != png::ColorType::Rgba
        || reader.info().bit_depth != png::BitDepth::Eight
    {
        return None;
    }
    let mut pixels = vec![0; reader.output_buffer_size()];
    let frame = reader.next_frame(&mut pixels).ok()?;
    let rgba = pixels[..frame.buffer_size()].to_vec();
    tao::window::Icon::from_rgba(rgba, frame.width, frame.height).ok()
}

pub fn run() -> Result<Infallible, Box<dyn std::error::Error>> {
    let blocker = adblock_engine::Blocker::from_embedded()?;
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(APP_TITLE)
        .with_window_icon(load_window_icon())
        .with_inner_size(tao::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build(&event_loop)?;
    let builder = WebViewBuilder::new()
        .with_url(SPOTIFY_URL)
        .with_autoplay(true)
        .with_devtools(true)
        .with_initialization_script(build_init_script())
        .with_navigation_handler(move |url| should_allow_navigation(&blocker, &url))
        .with_ipc_handler(|message| handle_ipc_message(message.body()));
    #[cfg(target_os = "linux")]
    let webview = {
        use tao::platform::unix::WindowExtUnix as _;
        use wry::WebViewBuilderExtUnix as _;
        builder.build_gtk(window.gtk_window())?
    };
    #[cfg(not(target_os = "linux"))]
    let webview = builder.build(&window)?;
    let _webview = webview;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        if matches!(
            event,
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            }
        ) {
            *control_flow = ControlFlow::Exit;
        }
    });
}

fn handle_ipc_message(body: &str) {
    match serde_json::from_str::<serde_json::Value>(body) {
        Ok(payload) => match payload.get("blocked").and_then(serde_json::Value::as_str) {
            Some(url) => tracing::warn!("page-level ad request blocked: {url}"),
            None => tracing::debug!("ipc message: {body}"),
        },
        Err(_) => tracing::debug!("non-JSON ipc message: {body}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_icon_valid() {
        assert!(load_window_icon().is_some());
    }
}
