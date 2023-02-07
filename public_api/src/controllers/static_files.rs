use super::*;

use rocket::http::ContentType;

#[get("/styles.css")]
pub async fn styles() -> (ContentType, &'static str) {
  (ContentType::CSS, include_str!("../../static/style.css"))
}

#[get("/bitcoin_libraries.js")]
pub async fn bitcoin_libraries() -> (ContentType, &'static str) {
  (ContentType::JavaScript, include_str!("../../static/bitcoin_libraries/dist/bundle.js"))
}
