mod server;
use server::*;

i18n::make_static_renderer!(RENDERER, "$CARGO_MANIFEST_DIR/src/assets");

fn render(lang: Lang, path: PathBuf, config: &State<Config>) -> LocalizedResult {
  Ok(RENDERER.i18n_and_serialize("public", lang, &path, config.inner())?)
}

#[launch]
async fn rocket() -> Rocket<Build> {
  server::rocket().await
}
