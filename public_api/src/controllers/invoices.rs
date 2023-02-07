use super::get;
use rocket::{
  response::Redirect,
};


#[get("/muchas-gracias")]
pub async fn muchas_gracias() -> Redirect {
  Redirect::permanent(format!("/#/invoices/muchas-gracias"))
}

#[get("/error-al-pagar")]
pub async fn error_al_pagar() -> Redirect {
  Redirect::permanent(format!("/#/invoices/error-al-pagar"))
}

#[get("/new?<link_token>")]
pub async fn new(mut link_token: String) -> Redirect {
  link_token = link_token.replace(" ", "+");
  Redirect::permanent(format!("/#/invoice/{}", link_token))
}