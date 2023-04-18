use tera::{self, Context};
use include_dir::{Dir, DirEntry};
use super::Lang;

#[macro_export]
macro_rules! make_static_renderer {
  ($(#[$attr:meta])* static ref $N:ident : $T:ty, $templates_dir:tt) => (
    static TEMPLATES_DIR: include_dir::Dir<'_> = include_dir::include_dir!($templates_dir);
    lazy_static::lazy_static! {
      $(#[$attr])* static ref $N:$T = { Renderer::new(&TEMPLATES_DIR) };
    }
  )
}

pub struct Renderer {
  pub templates: tera::Tera,
  pub template_names: Vec<String>,
}

impl Renderer {
  pub fn new(static_dir: &'static Dir<'static>) -> Self {
    let mut templates = tera::Tera::default();

    let mut entries: Vec<DirEntry> = static_dir
      .find("*").unwrap()
      .filter(|e| !file_name(e).starts_with('.') )
      .cloned().collect();

    entries.sort_by(|a,b|{
      let a_depth = path_name(a).matches("/").count();
      let b_depth = path_name(b).matches("/").count();
      let a_priority = file_name(a).starts_with("_");
      let b_priority = file_name(b).starts_with("_");

      if a_priority == b_priority {
        a_depth.cmp(&b_depth)
      } else {
        b_priority.cmp(&a_priority)
      }
    });

    for entry in entries {
      if let Some(file) = entry.as_file() {
        let pathname = path_name(&entry);

        if pathname.ends_with(".scss") {
          let style = grass::from_string(
            file.contents_utf8().expect(&format!("File is not utf-8: {}", pathname)).to_string(),
            &grass::Options::default().style(grass::OutputStyle::Compressed)
          ).expect(&format!("Failed to compile SCSS: {}", pathname));
          templates.add_raw_template(pathname, &style).expect("could not add template");
        } else {
          templates.add_raw_template(
            pathname,
            file.contents_utf8().expect(&format!("File is not utf-8: {}", pathname))
          ).expect("Could not add template");
        }
      }
    }

    let template_names = templates.get_template_names().map(|x| x.to_string() ).collect();
    Self { templates, template_names }
  }

  pub fn from_context(&self, lang: Lang, template_name: &str, ctx: &Context) -> tera::Result<String> {
    let local_template = format!("{}.{}", template_name, lang.code());
    let template = if self.template_names.contains(&local_template) { &local_template } else { template_name };
    self.templates.render(template, &ctx)
  }

  pub fn from_serialize<S: serde::Serialize>(&self, lang: Lang, template_name: &str, o: &S) -> tera::Result<String> {
    self.from_context(lang, template_name, &Context::from_serialize(o)?)
  }

  pub fn no_context(&self, lang: Lang, template_name: &str) -> tera::Result<String> {
    self.from_context(lang, template_name, &Context::new())
  }

  pub fn static_file(&self, template_name: &str) -> tera::Result<String> {
    self.templates.render(template_name, &Context::new())
  }
}

fn file_name<'a>(e: &'a DirEntry) -> &'a str {
  e.path().file_name().unwrap().to_str().unwrap()
}

fn path_name<'a>(e: &'a DirEntry) -> &'a str {
  e.path().to_str().unwrap()
}
