use super::get;
use rocket::response::Redirect;

#[get("/safe")]
pub async fn safe() -> Redirect {
  Redirect::permanent(format!("/#/safe"))
}

#[get("/safe/<token>")]
pub async fn prompt(token: String) -> Redirect {
  Redirect::permanent(format!("/#/safe/{token}"))
}

#[get("/safe/<token>/show")]
pub async fn show(token: String) -> Redirect {
  Redirect::permanent(format!("/#/safe/{token}/show"))
}