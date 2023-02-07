pub struct MagicLink;

impl MagicLink {
  pub fn make_random_token() -> String {
    use chbs::{config::BasicConfig, prelude::*};
    let mut config = BasicConfig::default();
    config.separator = "+".into();
    config.capitalize_first = false.into();
    config.to_scheme().generate()
  }
}
