use tera::{self, Context};
pub use include_dir::{Dir, DirEntry};
use std::collections::HashMap;
use super::{Lang, LocalizedResponse};
use std::path::{Path, PathBuf};
use std::io;
pub use rocket::http::ContentType;
use glob::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("File not found {0}")]
  NotFound(String),
  #[error("Unexpected error {0}")]
  Internal(String),
  #[error("An error ocurred while rendering error {0}")]
  Rendering(String),
}

impl From<glob::PatternError> for Error {
  fn from(_err: glob::PatternError) -> Error {
    Error::Internal("Invalid glob pattern".to_string())
  }
}

impl From<glob::GlobError> for Error {
  fn from(_err: glob::GlobError) -> Error {
    Error::Internal("Invalid glob operation".to_string())
  }
}

impl From<ex::io::Error> for Error {
  fn from(err: ex::io::Error) -> Error {
    Error::NotFound(format!("IO Error: {err:?}"))
  }
}

impl From<Box<grass::Error>> for Error {
  fn from(err: Box<grass::Error>) -> Error {
    Error::Rendering(format!("Error rendering stylesheet: {err:?}"))
  }
}

pub type RendererResult<T> = Result<T, Error>;

#[macro_export]
macro_rules! make_static_renderer {
  ($(#[$attr:meta])* static ref $N:ident : $T:ty, $templates_dir:tt) => (
    static TEMPLATES_DIR: include_dir::Dir<'_> = include_dir::include_dir!($templates_dir);
    lazy_static::lazy_static! {
      $(#[$attr])* static ref $N:$T = { Renderer::new(&TEMPLATES_DIR).unwrap() };
    }
  )
}

pub trait RendererFs: std::fmt::Debug {
  fn all_files(&self) -> RendererResult<Vec<PathBuf>>;
  fn is_file(&self, path: &Path) -> bool;
  fn read(&self, path: &Path) -> RendererResult<Vec<u8>>;
  fn read_to_string(&self, path: &Path) -> RendererResult<String>;
  fn render_style(&self, path: &Path) -> RendererResult<String>;
}

impl RendererFs for &'static Dir<'static> {
  fn all_files(&self) -> RendererResult<Vec<PathBuf>> {
    Ok(self.find("*")?.map(|e| e.path().to_owned() ).collect())
  }

  fn is_file(&self, path: &Path) -> bool {
    self.get_file(path).is_some()
  }

  fn read(&self, path: &Path) -> RendererResult<Vec<u8>> {
    self.get_file(path).map(|f| f.contents().to_vec() )
      .ok_or_else(|| Error::NotFound(path.display().to_string()))
  }

  fn read_to_string(&self, path: &Path) -> RendererResult<String> {
    self.get_file(path).and_then(|f| f.contents_utf8().map(|x| x.to_string()) )
      .ok_or_else(|| Error::NotFound(path.display().to_string()))
  }

  fn render_style(&self, path: &Path) -> RendererResult<String> {
    Ok(grass::from_string(
      self.read_to_string(path)?,
      &grass::Options::default().fs(&StaticGrassFs(self))
    )?)
  }
}

#[derive(Debug)]
pub struct StaticGrassFs(&'static Dir<'static>);

impl grass::Fs for StaticGrassFs {
  fn is_dir(&self, path: &Path) -> bool {
    self.0.get_dir(path).is_some()
  }

  fn is_file(&self, path: &Path) -> bool {
    self.0.is_file(path)
  }

  fn read(&self, path: &Path) -> Result<Vec<u8>, io::Error> {
    self.0.read(path).map_err(|_| io::Error::new(io::ErrorKind::NotFound, path.display().to_string()))
  }
}

impl RendererFs for &Path {
  fn all_files(&self) -> RendererResult<Vec<PathBuf>> {
    glob(&format!("{}/**/[!.]*", self.display()))?.map(|result|{
      result
        .map_err(|e| Error::Internal(format!("Glob result error on &str renderer")) )
        .and_then(|p|
          p.strip_prefix(self)
            .map(|p| p.to_path_buf())
            .map_err(|e| Error::Internal(format!("Could not strip prefix {}", self.display())))
        )
    }).collect::<Result<Vec<_>,_>>()
  }

  fn is_file(&self, path: &Path) -> bool {
    Path::is_file(&self.join(path))
  }

  fn read(&self, path: &Path) -> RendererResult<Vec<u8>> {
    Ok(ex::fs::read(self.join(path))?)
  }

  fn read_to_string(&self, path: &Path) -> RendererResult<String> {
    Ok(ex::fs::read_to_string(self.join(path))?)
  }

  fn render_style(&self, path: &Path) -> RendererResult<String> {
    Ok(grass::from_string(
      self.read_to_string(path)?,
      &grass::Options::default().fs(&DynamicGrassFs(self))
    )?)
  }
}
#[derive(Debug)]
pub struct DynamicGrassFs<'a>(&'a Path);

impl grass::Fs for DynamicGrassFs<'_> {
  fn is_dir(&self, path: &Path) -> bool {
    self.0.join(path).is_dir()
  }
  fn is_file(&self, path: &Path) -> bool {
    Path::is_file(&self.0.join(path))
  }

  fn read(&self, path: &Path) -> Result<Vec<u8>, io::Error> {
    std::fs::read(self.0.join(path))
  }
}

pub struct Renderer<FS> {
  pub htmls: tera::Tera,
  pub styles: HashMap<String, Vec<u8>>,
  pub fs: FS
}

impl<FS: RendererFs> Renderer<FS> {
  pub fn new(fs: FS) -> RendererResult<Self> {
    let mut htmls = tera::Tera::default();
    let mut styles = HashMap::new();

    let mut entries = fs.all_files()?;

    entries.sort_by(|a,b|{
      let a_depth = a.ancestors().count();
      let b_depth = b.ancestors().count();
      let a_priority = a.starts_with("_");
      let b_priority = b.starts_with("_");

      if a_priority == b_priority {
        a_depth.cmp(&b_depth)
      } else {
        b_priority.cmp(&a_priority)
      }
    });

    for entry in &entries {
      if !fs.is_file(entry) {
        continue;
      }

      let pathname = entry.display().to_string();

      if pathname.ends_with(".css") {
        let style = fs.render_style(entry)?;
        styles.insert(pathname, style.into_bytes());
      } else if pathname.ends_with(".html") {
        htmls.add_raw_template( &pathname, &fs.read_to_string(entry)?).expect("Could not add template");
      }
    }

    Ok(Self { htmls, styles, fs })
  }

  /* Try to serve a Vec or &[u8] here */
  pub fn render(&self, path: &str, ctx: &Context) -> Result<Vec<u8>, Error> {
    if path.ends_with(".html") {
      return self.htmls.render(path, ctx)
        .map(|t| t.into_bytes() )
        .map_err(|e| Error::Rendering(format!("Error rendering template {e:?}")) );
    }

    if path.ends_with(".css") {
      return self.styles.get(path).map(|x| x.to_owned() )
        .ok_or_else(|| Error::Rendering(format!("No style {path} found")) );
    }

    self.fs.read(Path::new(path))
  }

  pub fn render_localized(&self, prefix: &str, path: &PathBuf, lang: Lang, default_lang: Lang) -> Result<LocalizedResponse, Error> {
    let Some(ext) = path.extension().and_then(|x| x.to_str() ) else {
      return Err(Error::NotFound("Should have extension".to_string()))
    };

    let mime = match ext {
      "wasm" => ContentType::WASM,
      "ttf"  => ContentType::TTF,
      "png"  => ContentType::PNG,
      "js"   => ContentType::JavaScript,
      "css"  => ContentType::CSS,
      "scss"  => ContentType::CSS,
      "svg"  => ContentType::SVG,
      "html"  => ContentType::HTML,
      _ => return Err(Error::NotFound("No file found with that extension".to_string())),
    };

    let prefixed = Path::new(prefix);
    let lang_path = prefixed.join(lang.code()).join(path);
    let default_lang_path = prefixed.join(default_lang.code()).join(path);

    let (resolved_lang, resolved_path) = if self.fs.is_file(&lang_path) {
      (lang, lang_path)
    } else if self.fs.is_file(&default_lang_path) {
      (default_lang, default_lang_path)
    } else {
      (lang, prefixed.join(path))
    };

    let bytes = self.render(&resolved_path.display().to_string(), &Context::new())?;

    Ok(LocalizedResponse::new(bytes, mime, resolved_lang))
  }
}
