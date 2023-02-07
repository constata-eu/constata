use super::*;

#[get("/<id>")]
pub async fn show(id: i32, site: &State<Site>) -> JsonResult<Bulletin> {
  Ok(Json(site.bulletin().find(&id).await?))
}
