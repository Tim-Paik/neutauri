use std::{fs, io::Write};

mod data;

#[cfg(windows)]
const RUNTIME_DATA: &[u8] = include_bytes!("../../target/release/neutauri_runtime.exe");
#[cfg(not(windows))]
const RUNTIME_DATA: &[u8] = include_bytes!("../../target/release/neutauri_runtime");

#[cfg(not(windows))]
fn options() -> fs::OpenOptions {
    use std::os::unix::prelude::OpenOptionsExt;
    let mut options = fs::OpenOptions::new();
    options.write(true);
    options.create(true);
    options.truncate(true);
    options.mode(0o755);
    options
}

#[cfg(windows)]
fn options() -> fs::OpenOptions {
    let mut options = fs::OpenOptions::new();
    options.write(true);
    options.create(true);
    options.truncate(true);
    options
}

fn main() -> wry::Result<()> {
    let arg = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "neutauri.toml".into());
    if arg == "--help" || arg == "-h" {
        println!("Usage: neutauri_bundler [neutauri.toml]");
        return Ok(());
    }
    let config_path = std::path::Path::new(&arg).canonicalize()?;
    let config: data::Config = toml::from_str(fs::read_to_string(&config_path)?.as_str())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let source = match config_path.parent() {
        Some(path) => path.join(&config.source).canonicalize()?,
        None => config.source.canonicalize()?,
    };
    let target = match config_path.parent() {
        Some(path) => data::normalize_path(&path.join(&config.target)),
        None => data::normalize_path(&config.target),
    };
    let target = if target.extension() == None && cfg!(windows) {
        target.with_extension("exe")
    } else {
        target
    };
    if target.extension() == Some(std::ffi::OsStr::new("neu")) {
        data::pack(arg)?;
        return Ok(());
    }
    let data = data::Data::build_from_dir(source, config.window_attr()?, config.webview_attr()?)?;
    let mut f = options().open(&target)?;
    f.write_all(RUNTIME_DATA)?;
    f.write_all(&data)?;
    f.sync_all()?;
    f.flush()?;
    Ok(())
}
