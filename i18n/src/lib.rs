pub mod error;
pub mod renderer;
pub mod translations;
pub mod lang;
pub mod localized_response;

pub use tera::{self, Context, Tera, Result as TeraResult};
pub use grass;
pub use include_dir::{self, Dir, DirEntry};
pub use formatx;
pub use renderer::Renderer;
use std::borrow::Cow;
pub use lazy_static;
pub use lang::{Lang, MaybeLang};
pub use localized_response::LocalizedResponse;

use rocket::{
  self,
  response,
  http::{ContentType, hyper::header},
  request::{self, FromRequest, Outcome, Request},
};

#[macro_export]
macro_rules! t {
  ($lang:expr, $translation:ident) => {
    $lang.translations().$translation.to_string()
  };
  ($lang:expr, $translation:ident, $($e:tt)*) => {
    i18n::formatx::formatx!($lang.translations().$translation, $($e)*).expect("This can only fail if we mismatch translation params ourselves.")
  };
}
