mod server;
use server::*;

fn render(lang: Lang, path: PathBuf, config: &State<Config>) -> LocalizedResult {
  let renderer = Renderer::new(Path::new("src/assets/"))?;
  Ok(renderer.i18n_and_serialize("public", lang, &path, config.inner())?.into_owned())
}

#[launch]
async fn rocket() -> Rocket<Build> {
  server::rocket().await
}
