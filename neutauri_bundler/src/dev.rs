use anyhow::{Context, Result};
use neutauri_data as data;
use std::{fs, io::Read, path::PathBuf};
use wry::{
    application::{
        dpi::{PhysicalSize, Size},
        event::{Event, StartCause, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::{Fullscreen, Icon, Window, WindowBuilder},
    },
    webview::{WebContext, WebViewBuilder},
};

const PROTOCOL_PREFIX: &str = "{PROTOCOL}://";
const PROTOCOL: &str = "dev";

fn custom_protocol_uri<T: Into<String>>(protocol: T, path: T) -> String {
    PROTOCOL_PREFIX.replacen("{PROTOCOL}", &protocol.into(), 1) + &path.into()
}
fn custom_protocol_uri_to_path<T: Into<String>>(protocol: T, uri: T) -> wry::Result<String> {
    let prefix = PROTOCOL_PREFIX.replacen("{PROTOCOL}", &protocol.into(), 1);
    let uri = uri.into();
    let path = uri.strip_prefix(&prefix);
    match path {
        Some(str) => Ok(str.to_string()),
        None => Err(wry::Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            prefix + " is not found in " + &uri,
        ))),
    }
}

pub(crate) fn dev(config_path: String) -> Result<()> {
    let config_path = std::path::Path::new(&config_path)
        .canonicalize()
        .with_context(|| {
            format!(
                "Error reading config file from {}\n\n{}",
                &config_path, "You may want to create a neutauri.toml via the init subcommand?"
            )
        })?;
    let config: data::Config = toml::from_str(fs::read_to_string(&config_path)?.as_str())
        .with_context(|| "toml parsing error")?;
    let source = config.source.canonicalize()?;

    let event_loop = EventLoop::new();

    let window_builder = WindowBuilder::new()
        .with_always_on_top(config.window_attr()?.always_on_top)
        .with_decorations(config.window_attr()?.decorations)
        .with_resizable(config.window_attr()?.resizable)
        .with_title(config.window_attr()?.title)
        .with_maximized(config.window_attr()?.maximized)
        .with_transparent(config.window_attr()?.transparent)
        .with_visible(config.window_attr()?.visible);
    let window_builder = match config.window_attr()?.fullscreen {
        true => window_builder.with_fullscreen(Some(Fullscreen::Borderless(None))),
        false => window_builder,
    };
    let window_builder = match config.window_attr()?.icon {
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
    let window_builder = match config.window_attr()?.inner_size {
        Some(size) => window_builder.with_inner_size(get_size(size, monitor_size)),
        None => window_builder,
    };
    let window_builder = match config.window_attr()?.max_inner_size {
        Some(size) => window_builder.with_max_inner_size(get_size(size, monitor_size)),
        None => window_builder,
    };
    let window_builder = match config.window_attr()?.min_inner_size {
        Some(size) => window_builder.with_min_inner_size(get_size(size, monitor_size)),
        None => window_builder,
    };
    let window = window_builder.build(&event_loop)?;

    let webview_builder = WebViewBuilder::new(window)?;
    let url = config.webview_attr()?.url;
    let webview_builder = match url {
        Some(url) => {
            if url.starts_with('/') {
                webview_builder.with_url(&custom_protocol_uri(PROTOCOL, &url))?
            } else {
                webview_builder.with_url(&url)?
            }
        }
        None => webview_builder.with_url(&custom_protocol_uri(PROTOCOL, "/index.html"))?,
    };
    let html = config.webview_attr()?.html;
    let webview_builder = match html {
        Some(html) => webview_builder.with_html(&html)?,
        None => webview_builder,
    };
    let initialization_script = config.webview_attr()?.initialization_script;
    let webview_builder = match initialization_script {
        Some(script) => webview_builder.with_initialization_script(&script),
        None => webview_builder,
    };
    let webview_builder = match config.window_attr()?.visible {
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
        .with_visible(config.window_attr()?.visible)
        .with_transparent(config.window_attr()?.transparent)
        .with_web_context(&mut web_context)
        .with_custom_protocol(PROTOCOL.to_string(), move |request| {
            let path = custom_protocol_uri_to_path(PROTOCOL, request.uri())?;
            let mut local_path = source.clone();
            local_path.push(path.strip_prefix('/').unwrap_or(&path));
            let mut data = Vec::new();
            let mut mime: String = "application/octet-stream".to_string();
            match fs::File::open(&local_path) {
                Ok(mut f) => {
                    mime = new_mime_guess::from_path(&local_path)
                        .first_or_octet_stream()
                        .to_string();
                    f.read_to_end(&mut data)?;
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::NotFound && config.webview_attr()?.spa {
                        let mut index_path = source.clone();
                        index_path.push("index.html");
                        mime = new_mime_guess::from_path(&index_path)
                            .first_or_octet_stream()
                            .to_string();
                        let mut f = fs::File::open(index_path)?;
                        f.read_to_end(&mut data)?;
                    }
                }
            }
            wry::http::ResponseBuilder::new().mimetype(&mime).body(data)
        })
        .with_ipc_handler(|window: &Window, req: String| {
            match req.as_str() {
                "show_window" => window.set_visible(true),
                "ping" => println!("recived a ping"),
                _ => (),
            };
        })
        .with_devtools(true)
        .build()?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => webview.focus(),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
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
