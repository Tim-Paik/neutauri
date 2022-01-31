use std::{fs, io::Write};

mod data;

fn main() -> wry::Result<()> {
    let arg = std::env::args().nth(1).unwrap_or_else(|| "neutauri.toml".to_string());
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
    if target.extension() == Some(std::ffi::OsStr::new("neu")) {
        data::pack(arg)?;
        return Ok(());
    }
    let data = data::Data::build_from_dir(source, config.window_attr()?, config.webview_attr()?)?;
    let mut option = fs::OpenOptions::new();
    let option = option.write(true).create(true).truncate(true);
    let option = if cfg!(unix) {
        std::os::unix::prelude::OpenOptionsExt::mode(option, 0o755)
    } else {
        option
    };
    let mut f = option.open(&target)?;
    let runtime_data = include_bytes!("../../target/release/neutauri_runtime");
    f.write_all(runtime_data)?;
    f.write_all(&data)?;
    f.sync_all()?;
    f.flush()?;
    Ok(())
}
