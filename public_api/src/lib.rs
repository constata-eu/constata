/* TODO:
 * Mover todo graphql de src/controllers/certos/graphql a src/graphql
 * Renombrar todos los archivos de graphql.
 * Quitar los controllers del api anterior que ya no se usan.
 *    * Controllers tiene que ser solo de páginas server side rendered.
 *    * "web-app-certos" ahora es "webapp", la app de diplomas.
 *    * Ponemos dispatching dentro de constata para que cada pubkey tenga su experiencia propia.
 *
 * "certos app"
 */

#[macro_use]
#[cfg(any(test, feature = "test_support"))]
pub mod test_support;

pub use constata_lib::{
  error::{Error, Result},
  models::*,
  serde::{self, Deserialize, Serialize},
  signed_payload::SignedPayload,
};

pub use rocket::{self, fairing::AdHoc, get, post, routes, serde::json::Json, State};
use rocket::http::Method;
use rocket_cors::{AllowedHeaders, AllowedOrigins, Origins};
use rocket_recaptcha_v3::ReCaptcha;

i18n::make_static_renderer!(RENDERER, "$CARGO_MANIFEST_DIR/templates");

pub mod controllers;
use controllers::{
  static_files,
  bulletins,
  documents,
  stories,
  download_proof_links,
  explorer,
  pubkey_domain_endorsements,
  pubkeys,
  payments,
  account_state,
  terms_acceptance,
  template,
  request,
  entry,
  safe,
  invoices,
  public_certificates,
  vc_verifier,
  certos::{public_graphql::{
    new_graphql_schema,
    graphiql,
    get_graphql_handler,
    post_graphql_handler,
    introspect
  },
  certos_app
  }
};

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

  let schema = new_graphql_schema();

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
    .manage(schema)
    .attach(ReCaptcha::fairing())
    .attach(cors)
    .mount("/static", routes![
      static_files::styles,
      static_files::bitcoin_libraries,
    ])
    .mount("/payments", routes![
      payments::handle_stripe_events,
      payments::handle_btcpay_webhooks,
    ])
    .mount("/bulletins", routes![bulletins::show])
    .mount("/download-proof", routes![download_proof_links::show])
    .mount("/explorer", routes![explorer::show])
    .mount("/signup", routes![pubkeys::create])
    .mount(
      "/pubkey_domain_endorsements",
      routes![
        pubkey_domain_endorsements::index,
        pubkey_domain_endorsements::create,
      ],
    )
    .mount(
      "/stories",
      routes![
        stories::index,
        stories::create,
        stories::show,
        stories::html_proof,
      ],
    )
    .mount(
      "/documents",
      routes![
        documents::index,
        documents::create,
        documents::show,
        documents::html_proof,
        documents::each_part_html_proof,
      ],
    )
    .mount("/account_state", routes![account_state::show])
    .mount("/terms_acceptance",routes![
      terms_acceptance::show,
      terms_acceptance::show_bare,
      terms_acceptance::accept,
    ])
    .mount("/template", routes![
      template::download_payload,
    ])
    .mount("/request", routes![
      request::download_payload,
    ])
    .mount("/entry", routes![
      entry::download_payload,
    ])
    .mount("/invoices", routes![
      invoices::muchas_gracias, invoices::error_al_pagar, invoices::new
    ])
    .mount("/certificate", routes![
      public_certificates::show
    ])
    .mount("/graphql", routes![graphiql, get_graphql_handler, post_graphql_handler, introspect])
    .mount("/workroom", routes![
      certos_app::workroom_redirect,
    ])
    .mount("/vid_connect", routes![vc_verifier::callback])
    .mount("/", routes![
      safe::safe,
      safe::prompt,
      safe::show,
      certos_app::vid_chain_redirect_uri,
      certos_app::app,
      certos_app::build_dir,
    ])
}
