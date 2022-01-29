use wry::{
    application::{
        event::{Event, StartCause, WindowEvent},
        event_loop::{self, ControlFlow, EventLoop},
        window::{Fullscreen, Icon, Window, WindowBuilder},
    },
    webview::{RpcRequest, WebViewBuilder},
};
mod data;

#[cfg(windows)]
const PROTOCOL_PREFIX: &str = "https://{PROTOCOL}.";
#[cfg(not(windows))]
const PROTOCOL_PREFIX: &str = "{PROTOCOL}://";

const PROTOCOL: &str = "neu";

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

fn main() -> wry::Result<()> {
    data::pack("config.toml")?;
    let res = match data::load(std::env::current_exe()?.as_path()) {
        Ok(data) => data,
        Err(_) => data::load("app.neu")?,
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
    let window_builder = match res.window_attr.window_icon {
        Some(ref icon) => window_builder.with_window_icon(Some(Icon::from_rgba(
            icon.rgba.clone(),
            icon.width,
            icon.height,
        )?)),
        None => window_builder,
    };
    let monitor = event_loop
        .primary_monitor()
        .unwrap_or_else(|| event_loop.available_monitors().next().unwrap());
    dbg!(
        monitor.size(),
        monitor.name(),
        monitor.position(),
        monitor.scale_factor(),
        monitor.video_modes().collect::<Vec<_>>()
    );
    let window_builder = match res.window_attr.inner_size {
        Some(size) => match size {
            data::WindowSize::Large => todo!(),
            data::WindowSize::Medium => todo!(),
            data::WindowSize::Small => todo!(),
            data::WindowSize::Fixed(width, height) => todo!(),
        },
        None => window_builder,
    };
    let window_builder = match res.window_attr.max_inner_size {
        Some(size) => match size {
            data::WindowSize::Large => todo!(),
            data::WindowSize::Medium => todo!(),
            data::WindowSize::Small => todo!(),
            data::WindowSize::Fixed(width, height) => todo!(),
        },
        None => window_builder,
    };
    let window_builder = match res.window_attr.min_inner_size {
        Some(size) => match size {
            data::WindowSize::Large => todo!(),
            data::WindowSize::Medium => todo!(),
            data::WindowSize::Small => todo!(),
            data::WindowSize::Fixed(width, height) => todo!(),
        },
        None => window_builder,
    };
    let window = window_builder.build(&event_loop)?;

    let webview_builder = WebViewBuilder::new(window)?;
    let url = res.webview_attr.url.clone();
    let webview_builder = match url {
        Some(url) => {
            if url.starts_with("/") {
                webview_builder.with_url(&custom_protocol_uri(PROTOCOL, &url))?
            } else {
                webview_builder.with_url(&url)?
            }
        }
        None => webview_builder.with_url(&custom_protocol_uri(PROTOCOL, "/index.html"))?,
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
                r#"window.addEventListener('load', function(event) { rpc.call('show_window'); });"#,
            ),
    };
    let _webview = webview_builder
        .with_visible(res.window_attr.visible)
        .with_transparent(res.window_attr.transparent)
        .with_custom_protocol(PROTOCOL.to_string(), move |request| {
            let path = custom_protocol_uri_to_path(PROTOCOL, request.uri())?;
            let mut file = match res.open(path) {
                Ok(file) => file,
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::NotFound && res.webview_attr.spa {
                        res.open("index.html")?
                    } else {
                        return Err(wry::Error::Io(e));
                    }
                }
            };
            wry::http::ResponseBuilder::new()
                .mimetype(&file.mimetype())
                .body(file.decompressed_data()?)
        })
        .with_rpc_handler(|window: &Window, req: RpcRequest| {
            match req.method.as_str() {
                "show_window" => window.set_visible(true),
                "ping" => println!("recived a ping"),
                _ => (),
            };
            None
        })
        .build()?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
