#[macro_use]
#[cfg(any(test, feature = "test_support"))]
pub mod test_support;

pub use constata_lib::prelude::*;
use rocket::{self, fairing::AdHoc, routes, serde::json::Json, State};
use rocket::http::Method;
use rocket_cors::{AllowedHeaders, AllowedOrigins, Origins};
use rocket_recaptcha_v3::ReCaptcha;

pub mod api;
pub mod controllers;
pub mod current_person;
use api::*;
use controllers::*;
use current_person::*;

type JsonResult<T> = ConstataResult<Json<T>>;

i18n::make_static_renderer!(RENDERER, "$CARGO_MANIFEST_DIR/templates");

pub fn server(site: Site) -> rocket::Rocket<rocket::Build> {
  let allowed = AllowedOrigins::some(
    &[
      "https://api.audited.cloud",
      "https://api-staging.constata.eu",
      "https://api.constata.eu",
      "http://localhost:8000",
      "http://127.0.0.1:8000",
      "http://0.0.0.0:8000",
      "http://127.0.0.1:3000",
      "http://localhost:3000",
      "http://81.0.7.108",
    ],
    &["file://.*", "content://.*", "https://.*.constata.eu"]
  ).unwrap();

  let cors = rocket_cors::CorsOptions {
    allowed_origins: AllowedOrigins::Some(Origins{ allow_null: true, ..allowed}),
    allowed_methods: vec![Method::Get, Method::Post].into_iter().map(From::from).collect(),
    allowed_headers: AllowedHeaders::all(),
    allow_credentials: true,
    ..Default::default()
  }
  .to_cors().expect("Could not create cors.");

  rocket::build()
    .attach(AdHoc::on_ignite("site", |rocket| async move {
      rocket.manage(site)
    }))
    .attach(AdHoc::on_ignite("private_key", |rocket| async {
      let key = rocket.state::<Site>()
        .expect("site not loaded. This is an init bug on our end.")
        .keyring()
        .expect("could not init keyring to extract private key.")
        .expect("keyring is empty. Cannot init.")
        .private_key;
      rocket.manage(key)
    }))
    .manage(new_graphql_schema())
    .attach(ReCaptcha::fairing())
    .attach(cors)
    /*
    .mount("/static", routes![
      static_files::styles,
      static_files::bitcoin_libraries,
    ])
    */
    .mount("/payments", routes![
      payments::handle_stripe_events,
      payments::handle_btcpay_webhooks,
    ])
    .mount("/explorer", routes![explorer::show])
    .mount("/terms_acceptance",routes![
      terms_acceptance::show,
      terms_acceptance::show_bare,
      terms_acceptance::accept,
    ])
    .mount("/certificate", routes![
      public_certificates::show
    ])
    .mount("/graphql", routes![graphiql, get_handler, post_handler, introspect])
    .mount("/", routes![
      react_app::vid_chain_redirect_uri,
      react_app::app,
      react_app::build_dir,
    ])
}
