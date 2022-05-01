use bincode::Options;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Read, Result, Seek, SeekFrom},
    path::{self, Component, Path, PathBuf},
};
use wry::application::dpi::Position;

const MAGIC_NUMBER_START: &[u8; 9] = b"NEUTFSv01";
const MAGIC_NUMBER_END: &[u8; 9] = b"NEUTFSEnd";
const USIZE_LEN: usize = usize::MAX.to_be_bytes().len();

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Compress {
    Brotli,
    None,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct File {
    mime: String,
    data: Vec<u8>,
    compress: Compress,
}

#[derive(Serialize, Deserialize, Debug)]
struct Dir {
    files: Vec<(String, File)>,
    dirs: Vec<(String, Dir)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    pub window_attr: WindowAttr,
    pub webview_attr: WebViewAttr,
    fs: Dir,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum WindowSize {
    Large,
    Medium,
    Small,
    Fixed { width: f64, height: f64 },
    Scale { factor: f64 },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub source: PathBuf,
    pub target: PathBuf,
    pub inner_size: Option<WindowSize>,
    pub min_inner_size: Option<WindowSize>,
    pub max_inner_size: Option<WindowSize>,
    pub resizable: bool,
    pub fullscreen: bool,
    pub title: String,
    pub maximized: bool,
    pub visible: bool,
    pub transparent: bool,
    pub decorations: bool,
    pub always_on_top: bool,
    pub icon: Option<PathBuf>,
    pub spa: bool,
    pub url: Option<String>,
    pub html: Option<PathBuf>,
    pub initialization_script: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Icon {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WindowAttr {
    pub inner_size: Option<WindowSize>,
    pub min_inner_size: Option<WindowSize>,
    pub max_inner_size: Option<WindowSize>,
    pub position: Option<Position>,
    pub resizable: bool,
    pub fullscreen: bool,
    pub title: String,
    pub maximized: bool,
    pub visible: bool,
    pub transparent: bool,
    pub decorations: bool,
    pub always_on_top: bool,
    pub window_icon: Option<Icon>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WebViewAttr {
    pub visible: bool,
    pub transparent: bool,
    pub spa: bool,
    pub url: Option<String>,
    pub html: Option<String>,
    pub initialization_script: Option<String>,
}

impl File {
    pub fn decompressed_data(&mut self) -> Result<Vec<u8>> {
        let mut data = Vec::with_capacity(self.data.len());
        let mut r = brotli::Decompressor::new(self.data.as_slice(), 4096);
        r.read_to_end(&mut data)?;
        Ok(data)
    }
    pub fn mimetype(&self) -> String {
        self.mime.clone()
    }
}

impl Data {
    pub fn new<P: AsRef<path::Path> + Copy>(path: P) -> Result<Self> {
        let mut base = fs::File::open(path)?;
        let base_length = base.metadata()?.len();
        let mut magic_number_start_data = [0; MAGIC_NUMBER_START.len()];
        let mut data_length_data = [0; USIZE_LEN];
        let mut data = Vec::new();
        let mut magic_number_end_data = [0; MAGIC_NUMBER_END.len()];
        base.seek(SeekFrom::Start(base_length - MAGIC_NUMBER_END.len() as u64))?;
        // 此时指针指向 MAGIC_NUMBER_END 之前
        base.read_exact(&mut magic_number_end_data)?;
        if &magic_number_end_data != MAGIC_NUMBER_END {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "MAGIC_NUMBER_END not found",
            ));
        }
        base.seek(SeekFrom::Start(
            base_length - MAGIC_NUMBER_END.len() as u64 - USIZE_LEN as u64,
        ))?;
        // 此时指针指向 data_length 之前
        base.read_exact(&mut data_length_data)?;
        base.seek(SeekFrom::Start(
            base_length - u64::from_be_bytes(data_length_data),
        ))?;
        // 此时指针指向 MAGIC_NUMBER_START
        base.read_exact(&mut magic_number_start_data)?;
        if &magic_number_start_data != MAGIC_NUMBER_START {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "MAGIC_NUMBER_START not found",
            ));
        }
        base.read_exact(&mut data_length_data)?;
        // 此时指针指向 Data 前
        base.take(u64::from_be_bytes(data_length_data))
            .read_to_end(&mut data)?;
        let serialize_options = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .allow_trailing_bytes()
            .with_limit(104857600 /* 100MiB */);
        let fs: Self = match serialize_options.deserialize(&data) {
            Ok(fs) => fs,
            Err(e) => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, e));
            }
        };
        Ok(fs)
    }

    fn open_file(&self, current_dir: &Dir, mut path: path::Iter) -> Result<File> {
        let next_path = match path.next() {
            Some(str) => str.to_string_lossy().to_string(),
            None => return Err(io::Error::new(io::ErrorKind::NotFound, "file not found")),
        };
        for (name, file) in &current_dir.files {
            if next_path == *name {
                return Ok(file.clone());
            }
        }
        for (name, dir) in &current_dir.dirs {
            if next_path == *name {
                return self.open_file(dir, path);
            }
        }
        Err(io::Error::new(io::ErrorKind::NotFound, "file not found"))
    }

    pub fn open<P: AsRef<path::Path>>(&self, path: P) -> Result<File> {
        let path = normalize_path(path.as_ref());
        let path = if path.starts_with("/") {
            path.strip_prefix("/")
                .unwrap_or_else(|_| Path::new(""))
                .to_path_buf()
        } else {
            path
        };
        self.open_file(&self.fs, path.iter())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            source: PathBuf::from("."),
            target: PathBuf::from("app.neu"),
            inner_size: Some(WindowSize::Medium),
            min_inner_size: None,
            max_inner_size: None,
            resizable: true,
            fullscreen: false,
            title: "".into(),
            maximized: false,
            visible: true,
            transparent: false,
            decorations: true,
            always_on_top: false,
            icon: None,
            spa: false,
            url: Some("/index.html".into()),
            html: None,
            initialization_script: Some("".into()),
        }
    }
}

pub fn load<P: AsRef<path::Path> + Copy>(path: P) -> Result<Data> {
    Data::new(path)
}

pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => {}
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}
