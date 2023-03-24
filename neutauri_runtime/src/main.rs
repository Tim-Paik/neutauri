#![windows_subsystem = "windows"]

use neutauri_data as data;
use std::{borrow::Cow, path::PathBuf};
use wry::{
    application::{
        dpi::{PhysicalSize, Size},
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::{Fullscreen, Icon, Window, WindowBuilder},
    },
    webview::{WebContext, WebViewBuilder},
};

const PROTOCOL_PREFIX: &str = "neu://localhost";
const PROTOCOL: &str = "neu";

fn custom_protocol_uri<T: Into<String>>(path: T) -> String {
    PROTOCOL_PREFIX.to_owned() + &path.into()
}

fn main() -> wry::Result<()> {
    let res = match data::load(std::env::current_exe()?.as_path()) {
        Ok(data) => data,
        Err(_) => data::load("data.neu")?,
    };
    let event_loop = EventLoop::new();

    let window_builder = WindowBuilder::new()
        .with_always_on_top(res.window_attr.always_on_top)
        .with_decorations(res.window_attr.decorations)
        .with_resizable(res.window_attr.resizable)
        .with_title(res.window_attr.title.clone())
        .with_maximized(res.window_attr.maximized)
        .with_transparent(res.window_attr.transparent)
        .with_visible(res.window_attr.visible);
    let window_builder = match res.window_attr.fullscreen {
        true => window_builder.with_fullscreen(Some(Fullscreen::Borderless(None))),
        false => window_builder,
    };
    let window_builder = match res.window_attr.icon {
        Some(ref icon) => window_builder.with_window_icon(Some(Icon::from_rgba(
            icon.rgba.clone(),
            icon.width,
            icon.height,
        )?)),
        None => window_builder,
    };
    let monitor_size = event_loop
        .primary_monitor()
        .unwrap_or_else(|| {
            event_loop
                .available_monitors()
                .next()
                .expect("no monitor found")
        })
        .size();
    let window_builder = match res.window_attr.inner_size {
        Some(size) => window_builder.with_inner_size(get_size(size, monitor_size)),
        None => window_builder,
    };
    let window_builder = match res.window_attr.max_inner_size {
        Some(size) => window_builder.with_max_inner_size(get_size(size, monitor_size)),
        None => window_builder,
    };
    let window_builder = match res.window_attr.min_inner_size {
        Some(size) => window_builder.with_min_inner_size(get_size(size, monitor_size)),
        None => window_builder,
    };
    let window = window_builder.build(&event_loop)?;

    let webview_builder = WebViewBuilder::new(window)?;
    let url = res.webview_attr.url.clone();
    let webview_builder = match url {
        Some(url) => {
            if url.starts_with('/') {
                webview_builder.with_url(&custom_protocol_uri(&url))?
            } else {
                webview_builder.with_url(&url)?
            }
        }
        None => webview_builder.with_url(&custom_protocol_uri("/index.html"))?,
    };
    let html = res.webview_attr.html.clone();
    let webview_builder = match html {
        Some(html) => webview_builder.with_html(&html)?,
        None => webview_builder,
    };
    let initialization_script = res.webview_attr.initialization_script.clone();
    let webview_builder = match initialization_script {
        Some(script) => webview_builder.with_initialization_script(&script),
        None => webview_builder,
    };
    let webview_builder = match res.window_attr.visible {
        true => webview_builder.with_visible(true),
        false => webview_builder
            .with_visible(false)
            .with_initialization_script(
                r#"window.addEventListener('load', function(event) { window.ipc.postMessage('show_window'); });"#,
            ),
    };
    let path = std::env::current_exe()?;
    let path = path.file_stem().unwrap_or_else(|| "neutauri_app".as_ref());
    let mut web_context = if cfg!(target_os = "windows") {
        let config_path = match std::env::var("APPDATA") {
            Ok(dir) => PathBuf::from(dir),
            Err(_) => PathBuf::from("."),
        }
        .join(path);
        WebContext::new(Some(config_path))
    } else if cfg!(target_os = "linux") {
        let config_path = match std::env::var("XDG_CONFIG_DIR") {
            Ok(dir) => PathBuf::from(dir),
            Err(_) => match std::env::var("HOME") {
                Ok(dir) => PathBuf::from(dir).join(".config"),
                Err(_) => PathBuf::from("."),
            },
        }
        .join(path);
        WebContext::new(Some(config_path))
    } else if cfg!(target_os = "macos") {
        let config_path = match std::env::var("HOME") {
            Ok(dir) => PathBuf::from(dir).join("Library/Application Support/"),
            Err(_) => PathBuf::from("."),
        }
        .join(path);
        WebContext::new(Some(config_path))
    } else {
        WebContext::new(None)
    };
    let webview = webview_builder
        .with_clipboard(true)
        .with_visible(res.window_attr.visible)
        .with_transparent(res.window_attr.transparent)
        .with_web_context(&mut web_context)
        .with_initialization_script(
            r#"window.oncontextmenu = (event) => { event.preventDefault(); }"#,
        )
        .with_custom_protocol(PROTOCOL.to_string(), move |request| {
            let path = request.uri().path();
            let mut file = match res.open(path) {
                Ok(file) => file,
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::NotFound
                        && res.webview_attr.spa
                        && path != "/index.html"
                    {
                        res.open("index.html")?
                    } else {
                        return Err(wry::Error::Io(e));
                    }
                }
            };
            wry::http::Response::builder()
                .header("Content-Type", file.mimetype())
                .body(Cow::Owned(file.decompressed_data()?))
                .map_err(|e| e.into())
        })
        .with_ipc_handler(|window: &Window, req: String| {
            match req.as_str() {
                "show_window" => window.set_visible(true),
                "ping" => println!("recived a ping"),
                _ => (),
            };
        })
        .with_devtools(false)
        .build()?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::GlobalShortcutEvent(id) => webview
                .evaluate_script(&format!("GlobalShortcutEvent({:})", id.0))
                .unwrap_or_default(),
            _ => (),
        }
    });
}

fn get_size(size: data::WindowSize, monitor_size: PhysicalSize<u32>) -> Size {
    let (width, height) = match size {
        data::WindowSize::Large => (
            monitor_size.width as f64 * 0.7,
            monitor_size.height as f64 * 0.7,
        ),
        data::WindowSize::Medium => (
            monitor_size.width as f64 * 0.6,
            monitor_size.height as f64 * 0.6,
        ),
        data::WindowSize::Small => (
            monitor_size.width as f64 * 0.5,
            monitor_size.height as f64 * 0.5,
        ),
        data::WindowSize::Fixed { width, height } => (width, height),
        data::WindowSize::Scale { factor } => (
            monitor_size.width as f64 * factor,
            monitor_size.height as f64 * factor,
        ),
    };
    Size::Physical(PhysicalSize::new(width as u32, height as u32))
}
