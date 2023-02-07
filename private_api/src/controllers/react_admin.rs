use super::*;

use rocket::http::ContentType;

#[get("/")]
pub async fn app() -> (ContentType, &'static str) {
  (ContentType::HTML, include_str!("../../templates/index.html"))
}

#[get("/css/main.css")]
pub async fn css() -> (ContentType, &'static str) {
  (ContentType::CSS, include_str!("../../templates/static/css/main.css"))
}


#[get("/js/main.js")]
pub async fn javascript() -> (ContentType, &'static str) {
  (ContentType::JavaScript, include_str!("../../templates/static/js/main.js"))
}