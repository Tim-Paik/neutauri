use bincode::Options;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Read, Result},
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
    pub initialization_script: Option<PathBuf>,
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
    pub icon: Option<Icon>,
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

impl Dir {
    // 使用本地文件系统填充 Dir 结构体
    fn fill_with<P: AsRef<path::Path>>(
        &mut self,
        root: P,
        path: P,
        length: &mut u64,
    ) -> Result<()> {
        // 遍历目录
        for entry in fs::read_dir(path)? {
            let path = entry?.path();
            // 获取目录/文件名，如果为".."则跳过
            let name = match path.file_name() {
                Some(s) => match s.to_str() {
                    Some(s) => s.to_string(),
                    None => break,
                },
                None => break,
            };
            // 优先填充文件
            if path.is_file() {
                let mut source = fs::File::open(&path)?;
                let mime = new_mime_guess::from_path(&path)
                    .first_or_octet_stream()
                    .to_string();
                let mut data = brotli::CompressorReader::new(&mut source, 4096, 9, 21);
                let mut buffer = Vec::new();
                data.read_to_end(&mut buffer)?;
                let size = buffer.len();
                let file = File {
                    mime,
                    data: buffer,
                    compress: Compress::Brotli,
                };
                *length += size as u64;
                self.files.push((name, file));
            } else if path.is_dir() {
                // 构造子目录
                let mut dir = Dir {
                    files: Vec::new(),
                    dirs: Vec::new(),
                };
                // 填充子目录
                dir.fill_with(root.as_ref(), &path, length)?;
                self.dirs.push((name, dir));
            }
        }
        Ok(())
    }
}

impl Data {
    pub fn build_from_dir<P: AsRef<path::Path>>(
        source: P,
        window_attr: WindowAttr,
        webview_attr: WebViewAttr,
    ) -> Result<Vec<u8>> {
        let embed_fs = Self::from_dir(source, window_attr, webview_attr)?;
        embed_fs.build()
    }

    fn from_dir<P: AsRef<path::Path>>(
        source: P,
        window_attr: WindowAttr,
        webview_attr: WebViewAttr,
    ) -> Result<Self> {
        let source = source.as_ref();
        let mut length: u64 = 0;
        let mut dir = Dir {
            files: Vec::new(),
            dirs: Vec::new(),
        };
        dir.fill_with(source, source, &mut length)?;
        Ok(Self {
            fs: dir,
            window_attr,
            webview_attr,
        })
    }

    fn build(self) -> Result<Vec<u8>> {
        let serialize_options = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .allow_trailing_bytes()
            .with_limit(104857600 /* 100MiB */);
        // 构建文件头
        let data = match serialize_options.serialize(&self) {
            Ok(vec) => vec,
            Err(e) => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, e));
            }
        };

        let mut target: Vec<u8> = Vec::new();
        target.extend(MAGIC_NUMBER_START);
        target.extend(&data.len().to_be_bytes());
        target.extend(&data);
        let target_length = target.len();
        let target_length = target_length + USIZE_LEN;
        let target_length = target_length + MAGIC_NUMBER_END.len();
        target.extend(target_length.to_be_bytes());
        target.extend(MAGIC_NUMBER_END);

        Ok(target)
    }

    pub fn pack<P: AsRef<path::Path>>(config_path: P) -> Result<()> {
        let config_path = config_path.as_ref().canonicalize()?;
        let config: Config = toml::from_str(fs::read_to_string(&config_path)?.as_str())?;
        let source = match config_path.parent() {
            Some(path) => path.join(&config.source).canonicalize()?,
            None => config.source.canonicalize()?,
        };
        let target = match config_path.parent() {
            Some(path) => normalize_path(&path.join(&config.target)),
            None => normalize_path(&config.target),
        };
        fs::write(
            target,
            Self::build_from_dir(source, config.window_attr()?, config.webview_attr()?)?,
        )?;
        Ok(())
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
            initialization_script: None,
        }
    }
}

impl Config {
    pub fn window_attr(&self) -> Result<WindowAttr> {
        Ok(WindowAttr {
            inner_size: self.inner_size,
            min_inner_size: self.min_inner_size,
            max_inner_size: self.max_inner_size,
            position: None,
            resizable: self.resizable,
            fullscreen: self.fullscreen,
            title: self.title.clone(),
            maximized: self.maximized,
            visible: self.visible,
            transparent: self.transparent,
            decorations: self.decorations,
            always_on_top: self.always_on_top,
            icon: match &self.icon {
                Some(path) => Some(load_icon(path.as_path())?),
                None => None,
            },
        })
    }
    pub fn webview_attr(&self) -> Result<WebViewAttr> {
        Ok(WebViewAttr {
            visible: self.visible,
            transparent: self.transparent,
            spa: self.spa,
            url: self.url.clone(),
            html: match &self.html {
                Some(path) => fs::read_to_string(path.as_path()).ok(),
                None => None,
            },
            initialization_script: match &self.initialization_script {
                Some(path) => fs::read_to_string(path.as_path()).ok(),
                None => None,
            },
        })
    }
}

pub fn pack<P: AsRef<path::Path>>(config: P) -> Result<()> {
    Data::pack(config)
}

fn load_icon(path: &Path) -> Result<Icon> {
    let image = image::open(path)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
        .to_rgba8();
    Ok(Icon {
        width: image.dimensions().0,
        height: image.dimensions().1,
        rgba: image.into_raw(),
    })
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
            Component::Prefix(..) => unreachable!(),
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
