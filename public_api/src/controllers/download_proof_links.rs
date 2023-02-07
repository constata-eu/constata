use super::*;
use constata_lib::{ error::Result as ConstataResult, models::{blockchain::PrivateKey}};

use rocket::response::{self, Response, Responder};

pub struct DownloadableFile(String);

impl<'r> Responder<'r, 'static> for DownloadableFile {
  fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
    Response::build_from(self.0.respond_to(req)?)
      .raw_header("Content-Disposition", "attachment;filename=\"certificado.html\"")
      .ok()
  }
}

#[get("/<token>")]
pub async fn show(token: String, site: &State<Site>, key: &State<PrivateKey>, l: Lang) -> ConstataResult<DownloadableFile> {
  Ok(DownloadableFile(
    site.download_proof_link()
      .active(token)
      .one().await?
      .html_proof(&key, l).await?
  ))
}

constata_lib::describe_one! {

  macro_rules! setup_doc_and_link {
    ($client:ident, $chain:ident, $expired:expr) => (
      $chain.fund_signer_wallet();
      let document = $client.signer.signed_document(&b"Hello World!"[..]).await;
      $chain.simulate_stamping().await;

      let download_proof_link = $client.signer.make_download_proof_link_from_doc(&document, 1).await;
      download_proof_link.access_token().await?.update().token("thisIsATest".to_string()).save().await?;
      if $expired {
        download_proof_link.access_token().await?.update().expired(true).save().await?;
      }
    )
  }

  fulltest!{ can_download_a_proof (_site, c, client, mut chain)
    setup_doc_and_link!(client, chain, false);
    assert_that!(
      &client.get_string("/download-proof/thisIsATest").await,
      rematch("voy a ayudarte a entender como se valida este sello de tiempo")
    );
  }

  fulltest!{ can_download_a_proof_in_english (_site, c, mut client, mut chain)
    client.set_lang(i18n::Lang::En);
    setup_doc_and_link!(client, chain, false);
    assert_that!(
      &client.get_string("/download-proof/thisIsATest").await,
      rematch("I'm a constata developer")
    );
  }

  fulltest!{ cant_download_with_expired_link (_site, c, client, mut chain)
    setup_doc_and_link!(client, chain, true);
    client.assert_get_error("/download-proof/thisIsATest", Status::NotFound, "Not found").await;
  }

  fulltest!{ cant_download_with_unknown_link (_site, c, client, mut chain)
    setup_doc_and_link!(client, chain, false);
    client.assert_get_error("/download-proof/badtoken", Status::NotFound, "Not found").await;
  }

  fulltest!{ cannot_access_after_org_deletion (_site, c, client, mut chain)
    setup_doc_and_link!(client, chain, false);
    client.signer.make_org_deletion(b"person deletion").await;

    let response = client.get_response("/download-proof/thisIsATest").await;
    assert_eq!(response.status(), Status::NotFound);
  }
}
