use wry::{
    application::{
        event::{Event, StartCause, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    },
    webview::WebViewBuilder,
};
mod data;

#[cfg(windows)]
const PROTOCOL_PREFIX: &str = "https://{PROTOCOL}.";
#[cfg(not(windows))]
const PROTOCOL_PREFIX: &str = "{PROTOCOL}://";

const PROTOCOL: &str = "neutauri";

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
    let res = data::load(std::env::current_exe()?.as_path())?;
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop)?;
    let _webview = WebViewBuilder::new(window)?
        .with_transparent(false)
        .with_custom_protocol(PROTOCOL.to_string(), move |request| {
            let path = custom_protocol_uri_to_path(PROTOCOL, request.uri())?;
            let mut file = res.open(path)?;
            wry::http::ResponseBuilder::new()
                .mimetype(&file.mimetype())
                .body(file.decompressed_data()?)
        })
        .with_url(&custom_protocol_uri(PROTOCOL, "/index.html"))?
        .build()?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => println!("Wry has started!"),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
