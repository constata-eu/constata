pub mod app;
pub mod template;
pub mod request;
pub mod entry;
pub mod template_kind;
pub mod wizard;

pub use app::*;
pub use template::*;
pub use request::*;
pub use entry::*;
pub use template_kind::*;
pub use wizard::*;

describe!{
  use std::io::Read; 
  use bitcoin::network::constants::Network;
  use crate::models::{Previewer, storable::*};

  regtest!{ sets_up_certos_and_processes_one_request (site, c, mut chain)
    let alice = c.alice().await;
    let email = alice.make_email("alice@example.com").await;
    site.email_address().verify_with_token(&email.access_token().await?.unwrap()).await?;

    let mut request = set_up_request(
      &alice,
      "src/test_support/resources/certos_request.csv"
    ).await?;
    assert!(request.entry_vec().await?.is_empty());
    
    site.request().create_all_received().await?; // Ahora se crean todos los documentos.
    request.reload().await?;

    let export_received = read_to_string("certos_request_export_received.csv");
    assert_that!(&request.export_csv().await?, rematch(&export_received));

    let created_entries = request.entry_vec().await?.into_iter()
      .map(|a| a.in_created() )
      .collect::<crate::Result<Vec<entry::Created>>>()?;

    assert_eq!(created_entries.len(), 2);

    let payload = created_entries[0].as_inner().payload().await?;
    let mut zipfile = zip::ZipArchive::new(std::io::Cursor::new(&payload))?;

    {
      let inner_0 = zipfile.by_index(0).unwrap();
      assert_eq!(inner_0.name(), "3_mensaje.html");
    }

    {
      let inner_1 = zipfile.by_index(1).unwrap();
      assert_eq!(inner_1.name(), "2_analítico.html");
    }

    {
      let mut inner_2 = zipfile.by_index(2).unwrap();
      assert_eq!(inner_2.name(), "1_diploma.html");
      let mut contents_2 = String::new();
      inner_2.read_to_string(&mut contents_2).unwrap();
      assert_that!(&contents_2, rematch("Derecho Épico"));
      assert_that!(&contents_2, rematch("22</strong> de <strong>marzo"));
    }

    let html_preview = Previewer::create(&payload, true)?.render_html(i18n::Lang::Es)?;
    std::fs::write("../target/artifacts/entry_preview_es_kyc.html", &html_preview)?;
    let html_preview = Previewer::create(&payload, false)?.render_html(i18n::Lang::Es)?;
    std::fs::write("../target/artifacts/entry_preview_es_no_kyc.html", &html_preview)?;

    let html_preview = Previewer::create(&payload, true)?.render_html(i18n::Lang::En)?;
    std::fs::write("../target/artifacts/entry_preview_en_kyc.html", &html_preview)?;
    let html_preview = Previewer::create(&payload, false)?.render_html(i18n::Lang::En)?;
    std::fs::write("../target/artifacts/entry_preview_en_no_kyc.html", &html_preview)?;

    let created = request.in_created()?;
    let mut signature = None;
    while let Some(next) = created.signing_iterator(signature).await? {
      signature = Some(alice.sign_request_entry(next).await);
    }

    request.reload().await?;
    assert!(request.is_signed());

    // Now all documents should be accepted.
    for e in &request.entry_vec().await? {
      let doc = e.in_signed()?.document().await?;
      assert!(doc.is_accepted());
    }
    assert!(request
      .entry_vec().await?[0]
      .in_signed()?
      .document().await?
      .email_callback_vec().await?[0]
      .sent_at().is_none()
    );

    chain.fund_signer_wallet();
    chain.simulate_stamping().await;

    site.request().try_complete().await?;
    request.reload().await?;
    assert!(!request.is_completed());
    assert!(request.entry_vec().await?[0].is_signed());
    assert!(request.entry_vec().await?[1].is_completed());
   
    let key = TestBlockchain::default_private_key().await?;
    let proof = request.entry_vec().await?[1].in_completed()?.document().await?.story().await?.proof(Network::Regtest, &key).await?;
    let content = proof.render_html(i18n::Lang::Es).expect("Content to be ready now");
    std::fs::write("../target/artifacts/diploma_camara.html", &content)?;

    let doc = &request.entry_vec().await?[0].in_signed()?.document().await?.in_accepted()?;
    assert!(doc.bulletin().await?.is_published());
    let callback = doc.as_inner().email_callback_vec().await?.pop().unwrap();
    callback.clone().mark_sent().await?;
    let email = callback.render_mailer_html().await?;
    assert_that!(&email, rematch("La empresa Constata.EU le transmite este mensaje"));

    site.request().try_complete().await?;
    request.reload().await?;
    assert!(request.is_completed());
    for e in &request.entry_vec().await? {
      assert!(e.is_completed());
    }

    let expected = read_to_string("certos_request_export_done.csv");
    assert_that!(&request.export_csv().await?, rematch(&expected));
  }

  dbtest!{ accepts_csv_with_semicolon (site, c)
    let alice = c.alice().await;
    let request = set_up_request(
      &alice,
      "src/test_support/resources/certos_request_semicolon.csv"
    ).await?;
    site.request().create_all_received().await?;
    let entries = request.entry_vec().await?;
    assert_eq!(2, entries.len());
    assert_that!(entries[0].params(), rematch("\"curso\":\"Derecho\""));
  }

  regtest!{ bad_certos_request_unequal_lengths (_site, c, _chain)
    set_up_request(
      &c.alice().await,
      "src/test_support/resources/bad_certos_request_unequal_lengths.csv"
    ).await.unwrap_err();
  }

  regtest!{ bad_certos_request_non_ascii_character (_site, c, _chain)
    set_up_request(
      &c.alice().await,
      "src/test_support/resources/bad_certos_request_non_ascii_character.csv"
    ).await.unwrap_err();
  }

  regtest!{ bad_certos_request_incompatible_with_template (site, c, _chain)
    let mut request = set_up_request(
      &c.alice().await,
      "src/test_support/resources/bad_certos_request_incompatible_with_template.csv"
    ).await?;
    
    assert!(request.entry_vec().await?.is_empty());

    site.request().create_all_received().await.unwrap_err(); // Ahora se crean todos los documentos.

    request.reload().await?;
    assert!(request.is_failed());
    assert!(request.entry_vec().await?.is_empty());
  }

  dbtest!{ accepts_crlf_requests (site, c)
    let request = set_up_request(
      &c.alice().await,
      "src/test_support/resources/certos_request_crlf.csv"
    ).await.expect("Valid request");
    site.request().create_all_received().await?;
    assert_that!(&request.entry_vec().await?[1].attrs.params, rematch("Matemáticas"));
  }

  regtest!{ user_discards_request (site, c, _chain)
    let request = set_up_request(
      &c.alice().await,
      "src/test_support/resources/certos_request.csv"
    ).await?;
    
    assert!(request.entry_vec().await?.is_empty());

    site.request().create_all_received().await?;

    let created = request.reloaded().await?.in_created()?;
    let failed = created.discard().await?;
    assert_eq!(failed.errors(), "user_discarded");
  }
  
  dbtest!{ use_template_file_with_image (site, c)
    let alice = c.alice().await;
    let template_file = read("certos_template_with_image.zip");
    let template = alice.make_template(template_file).await;
    let request_file = read("certos_request.csv");
    alice.make_request(*template.id() ,request_file).await?;

    site.request().create_all_received().await?;
    let templates_files = Template::read_name_and_bytes_from_payload(&template.storage_fetch().await?).await?;
    assert_eq!(templates_files.len(), 4)
  }

  async fn set_up_request(alice: &SignerClient, request_path: &str) -> crate::Result<Request> {
    let template_file = read("certos_template.zip");
    let template = alice.make_template(template_file).await;
    let request_file = std::fs::read(request_path)?;
    alice.make_request(*template.id() ,request_file).await
  }
}
