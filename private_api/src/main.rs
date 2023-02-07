#[macro_use]
#[cfg(test)]
pub mod test_support;

pub use rocket::{self, fairing::AdHoc, get, post, routes, State};

use constata_lib::Site;

mod controllers;
use controllers::*;

use controllers::private_graphql::{
  new_graphql_schema,
  graphiql,
  get_graphql_handler,
  post_graphql_handler,
};

use rocket::http::Method;
use rocket_cors::{AllowedHeaders, AllowedOrigins};

#[rocket::launch]
async fn rocket() -> rocket::Rocket<rocket::Build> {
  // It's ok to unwrap here as it will panic when the process launches, which helps us know and fix it right away.
  server(Site::from_stdin_password().await.unwrap())
}

fn server(site: Site) -> rocket::Rocket<rocket::Build> {
  // You can also deserialize this
  let cors = rocket_cors::CorsOptions {
      allowed_origins: AllowedOrigins::some_exact(&[
        "https://4dm1n.audited.cloud",
        "https://4dm1n-staging.constata.eu",
        "https://4dm1n.constata.eu",
        "http://localhost:8000",
        "http://127.0.0.1:8000",
        "http://127.0.0.1:3000",
        "http://localhost:3000",
      ]),
      allowed_methods: vec![Method::Get, Method::Post].into_iter().map(From::from).collect(),
      allowed_headers: AllowedHeaders::all(),
      allow_credentials: true,
      ..Default::default()
  }
  .to_cors().expect("No pude crear el CORS.");

  rocket::build()
    .attach(AdHoc::on_ignite("sqlx", |rocket| async move {
      rocket.manage(site)
    }))
    .manage(new_graphql_schema())
    .attach(cors)
    .mount("/sessions", routes![sessions::create])
    .mount("/graphql", rocket::routes![graphiql, get_graphql_handler, post_graphql_handler])
    .mount("/admin", rocket::routes![react_admin::app])
    .mount("/static", routes![
      react_admin::css,
      react_admin::javascript
      ])
}
