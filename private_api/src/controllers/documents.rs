use super::*;
use constata_lib::models::document::*;

#[get("/?<q>")]
pub async fn index(_key: ApiKey, site: &State<Site>, q: DocumentQuery) -> JsonResult<Vec<Document>> {
  Ok(Json(Document::all(&site, q).await?))
}

#[get("/<id>")]
pub async fn show(_key: ApiKey, id: String, site: &State<Site>) -> JsonResult<Document> {
  Ok(Json(Document::find_by_id(&site, &id).await?))
}

constata_lib::describe_one! {
  apitest!{ lists_and_retrieves_documents_using_filters (_db, c, client)
    let one = c.alice().await.signed_document(&b"Hello World!"[..]).await;
    let two = c.bob().await.signed_document(&b"Hello World!"[..]).await;
    let docs: Vec<Document> = client.get(format!("/documents/?q.person_id={}", one.person_id())).await;
    assert_eq!(docs, vec![one]);

    let doc: Document = client.get(format!("/documents/{}", two.id())).await;
    assert_eq!(doc, two);
  }
}
