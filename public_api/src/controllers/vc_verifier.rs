use super::*;

#[post("/callback", data="<credential>")]
pub async fn callback(credential: &str) -> Result<String> {
  Ok("ok".to_string())
}
