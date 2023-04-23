use tera::{self, Context};
use std::borrow::Cow;
pub use include_dir::{Dir, DirEntry};
use std::collections::HashMap;
use super::{Lang, LocalizedResponse};
use std::path::{Path, PathBuf};
use std::io;
pub use rocket::http::ContentType;
use glob::*;
use super::error::{Error, RendererResult};

#[macro_export]
macro_rules! make_static_renderer {
  ($N:ident, $templates_dir:tt) => (
    static TEMPLATES_DIR: i18n::include_dir::Dir<'_> = i18n::include_dir::include_dir!($templates_dir);
    i18n::lazy_static::lazy_static! {
      static ref $N: i18n::renderer::Renderer<&'static i18n::include_dir::Dir<'static>> = { i18n::renderer::Renderer::new(&TEMPLATES_DIR).unwrap() };
    }
  )
}

pub trait RendererFs: std::fmt::Debug {
  fn all_files(&self) -> RendererResult<Vec<PathBuf>>;
  fn is_file(&self, path: &Path) -> bool;
  fn read(&self, path: &Path) -> RendererResult<Cow<[u8]>>;
  fn read_to_string(&self, path: &Path) -> RendererResult<Cow<str>>;
  fn render_style(&self, path: &Path) -> RendererResult<String>;
}

impl RendererFs for &'static Dir<'static> {
  fn all_files(&self) -> RendererResult<Vec<PathBuf>> {
    Ok(self.find("*")?.map(|e| e.path().to_owned() ).collect())
  }

  fn is_file(&self, path: &Path) -> bool {
    self.get_file(path).is_some()
  }

  fn read(&self, path: &Path) -> RendererResult<Cow<[u8]>> {
    self.get_file(path).map(|f| Cow::Borrowed(f.contents()) )
      .ok_or_else(|| Error::NotFound(path.display().to_string()))
  }

  fn read_to_string(&self, path: &Path) -> RendererResult<Cow<str>> {
    self.get_file(path).and_then(|f| f.contents_utf8().map(|x| Cow::Borrowed(x) ) )
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
    self.0.get_file(path)
      .map(|f| f.contents().to_vec() )
      .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, path.display().to_string()))
  }
}

impl RendererFs for &Path {
  fn all_files(&self) -> RendererResult<Vec<PathBuf>> {
    let mut all = vec![];

    for result in glob(&format!("{}/**/[!.]*", self.display()))? {
      all.push(result?.strip_prefix(self)?.to_path_buf());
    }

    Ok(all)
  }

  fn is_file(&self, path: &Path) -> bool {
    Path::is_file(&self.join(path))
  }

  fn read(&self, path: &Path) -> RendererResult<Cow<[u8]>> {
    Ok(Cow::Owned(ex::fs::read(self.join(path))?))
  }

  fn read_to_string(&self, path: &Path) -> RendererResult<Cow<str>> {
    Ok(Cow::Owned(ex::fs::read_to_string(self.join(path))?))
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
    htmls.autoescape_on(vec![]);

    let mut styles = HashMap::new();

    let mut entries = fs.all_files()?;

    entries.sort_by(|a,b|{
      let a_priority = Self::priority(a);
      let b_priority = Self::priority(b);

      if a_priority == b_priority {
        a.ancestors().count().cmp(&b.ancestors().count())
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
        htmls.add_raw_template( &pathname, &style).expect("Could not add template");
        styles.insert(pathname, style.into_bytes());
      } else if pathname.ends_with(".html") {
        htmls.add_raw_template( &pathname, &fs.read_to_string(entry)?).expect("Could not add template");
      }
    }


    Ok(Self { htmls, styles, fs })
  }

  fn priority(path: &Path) -> bool {
    path.file_name().and_then(|f| f.to_str()).map(|f| f.starts_with("_") ).unwrap_or(false)
  }
  
  pub fn render(&self, path: &str, ctx: &Context) -> RendererResult<Cow<[u8]>> {
    if path.ends_with(".html") {
      return Ok(self.htmls.render(path, ctx).map(|t| Cow::Owned(t.into_bytes()) )?);
    }

    if path.ends_with(".css") {
      return self.styles.get(path).map(|x| Cow::Borrowed(x.as_slice()) )
        .ok_or_else(|| Error::NotFound(format!("No style {path} found")) );
    }

    self.fs.read(Path::new(path))
  }

  pub fn render_localized_context(&self, prefix: &str, path: &PathBuf, lang: Lang, default: Lang, c: &Context) -> RendererResult<LocalizedResponse> {
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
    let default_lang_path = prefixed.join(default.code()).join(path);

    let (resolved_lang, resolved_path) = if self.fs.is_file(&lang_path) {
      (lang, lang_path)
    } else if self.fs.is_file(&default_lang_path) {
      (default, default_lang_path)
    } else {
      (lang, prefixed.join(path))
    };

    let bytes = self.render(&resolved_path.display().to_string(), c)?;

    Ok(LocalizedResponse::new(bytes, mime, resolved_lang))
  }

  pub fn render_localized_and_serialized<S: serde::Serialize>(&self, prefix: &str, path: &PathBuf, lang: Lang, default: Lang, c: S) 
    -> RendererResult<LocalizedResponse>
  {
    self.render_localized_context(prefix, path, lang, default, &Context::from_serialize(&c)?)
  }

  pub fn render_localized(&self, prefix: &str, path: &PathBuf, lang: Lang, default: Lang) -> RendererResult<LocalizedResponse> {
    self.render_localized_context(prefix, path, lang, default, &Context::new())
  }
}
