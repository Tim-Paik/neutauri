use crate::data;
use std::{fs, io::Write};

#[cfg(windows)]
const RUNTIME_DATA: &[u8] = include_bytes!("../../target/release/neutauri_runtime.exe");
#[cfg(not(windows))]
const RUNTIME_DATA: &[u8] = include_bytes!("../../target/release/neutauri_runtime");

fn options() -> fs::OpenOptions {
    #[cfg(not(windows))]
    use std::os::unix::prelude::OpenOptionsExt;
    let mut options = fs::OpenOptions::new();
    options.write(true);
    options.create(true);
    options.truncate(true);
    #[cfg(not(windows))]
    options.mode(0o755);
    options
}

pub fn bundle(config_path: String) -> std::io::Result<()> {
    let config_path = std::path::Path::new(&config_path).canonicalize()?;
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
        data::pack(config_path)?;
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
