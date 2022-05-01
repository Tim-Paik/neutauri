use crate::data;
use std::{
    env, fs,
    hash::{Hash, Hasher},
    io::{self, Write},
};

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

#[cfg(not(windows))]
fn get_runtime_data() -> io::Result<Vec<u8>> {
    Ok(include_bytes!("../../target/release/neutauri_runtime").to_vec())
}

#[cfg(windows)]
fn get_runtime_data() -> io::Result<Vec<u8>> {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hasher.write(b"neutauri_runtime");
    std::time::SystemTime::now().hash(&mut hasher);
    println!("Hash is {:x}!", hasher.finish());
    let temp_path = env::temp_dir().join(format!("{:x}.exe", hasher.finish()));
    fs::write(
        &temp_path,
        include_bytes!("../../target/release/neutauri_runtime.exe"),
    )?;
    // let mut updater = rcedit::ResourceUpdater::new();
    // updater.load(&temp_path).unwrap(); // TODO: handle error
    fs::read(&temp_path)
}

pub fn bundle(config_path: String) -> io::Result<()> {
    let config_path = std::path::Path::new(&config_path).canonicalize()?;
    let config: data::Config = toml::from_str(fs::read_to_string(&config_path)?.as_str())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let source = match config_path.parent() {
        Some(path) => path.join(&config.source).canonicalize()?,
        None => config.source.canonicalize()?,
    };
    let target = match config_path.parent() {
        Some(path) => data::normalize_path(&path.join(&config.target)),
        None => data::normalize_path(&config.target),
    };
    fs::create_dir_all(target.parent().unwrap_or_else(|| std::path::Path::new("/")))?;
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
    f.write_all(&get_runtime_data()?)?;
    f.write_all(&data)?;
    f.sync_all()?;
    f.flush()?;
    Ok(())
}
