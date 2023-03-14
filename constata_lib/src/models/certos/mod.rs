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
  use std::collections::HashMap;
  use bitcoin::network::constants::Network;
  use crate::{
    models::{Previewer, storable::*},
    Result as ConstataResult,
    Error
  };

  regtest!{ process_one_issuance_from_an_arbitrary_template (site, c, mut chain)
    let alice = c.alice().await;
    let email = alice.make_email("alice@example.com").await;
    site.email_address().verify_with_token(&email.access_token().await?.unwrap()).await?;

    let mut request = set_up_csv_request_with_custom_template(
      &alice,
      "src/test_support/resources/certos_request.csv"
    ).await?;
    assert!(request.flow().is_received());
    assert_eq!(request.entry_scope().state_eq("received".to_string()).count().await?, 2);
    
    site.request().create_all_received().await?; // Ahora se crean todos los documentos.
    request.reload().await?;

    let export_received = read_to_string("certos_request_export_received.csv");
    assert_that!(&request.export_csv().await?, rematch(&export_received));

    let created_entries = request.entry_vec().await?.into_iter()
      .map(|a| a.in_created() )
      .collect::<ConstataResult<Vec<entry::Created>>>()?;

    assert_eq!(created_entries.len(), 2);

    let payload = created_entries[0].as_inner().payload().await?;
    let mut zipfile = zip::ZipArchive::new(std::io::Cursor::new(&payload))?;

    {
      let mut inner_0 = zipfile.by_index(0).unwrap();
      assert_eq!(inner_0.name(), "1_diploma.html");
      let mut contents_0 = String::new();
      inner_0.read_to_string(&mut contents_0).unwrap();
      assert_that!(&contents_0, rematch("Lisa Simpson"));
    }

    {
      let inner_2 = zipfile.by_index(1).unwrap();
      assert_eq!(inner_2.name(), "2_analítico.html");
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

  dbtest!{ submits_wizard_with_new_template(site, c)
    let a = c.alice().await;
    let person = a.person().await;

    let w = Wizard{
      person,
      name: "Some diploma 2023".to_string(),
      template: WizardTemplate::New {
        name: "A diploma template".to_string(),
        logo: ImageOrText::Image(read("wizard/logo.png")),
        kind: TemplateKind::Invitation,
      },
      csv: read("wizard/default.csv"),
    };

    let request = w.process().await?;
    site.request().create_all_received().await?;

    assert_eq!(
      &request.entry_vec().await?[0].params_and_custom_message().await?.1.unwrap(),
      "Hola Stan Marsh, esta es una invitación para el evento llamado Arte con plastilina."
    );

    assert_eq!(request.entry_scope().count().await?, 2);
    std::fs::write("../target/artifacts/entry.zip", &request.entry_scope().one().await?.payload().await?)?;


    let mut zipfile = zip::ZipArchive::new(std::io::Cursor::new(request.template().await?.payload().await?))?;
    let mut inner = zipfile.by_index(0)?;
    assert_eq!(inner.name(), "invitation.html.tera");
    let mut contents = String::new();
    inner.read_to_string(&mut contents)?;
    assert_that!(&contents, rematch(r#"\{\{ name \}\}"#));
    assert_that!(&contents, rematch("data:image/png;base64,iVBORw0KG"));

    std::fs::write("../target/artifacts/template_from_submits_wizard_with_new_template.html", &contents)?;
  }

  dbtest!{ builds_and_submits_issuance_from_json(site, c)
    let a = c.alice().await;
    let person = a.person().await;

    let w = JsonIssuanceBuilder {
      person,
      name: "Some diploma 2023".to_string(),
      template: WizardTemplate::New {
        name: "A diploma template".to_string(),
        logo: ImageOrText::Text("SuperUni".to_string()),
        kind: TemplateKind::Invitation,
      },
      entries: serde_json::from_value(serde_json::json!([
        {
          "name": "Stan Marsh",
          "email": "stan@cc.com",
          "recipient_identification": "AB-12345",
          "custom_text": "Artísta plastilínico",
          "motive": "Arte con plastilina",
          "date": "3 de marzo de 1999",
          "place": "Sout Park",
          "shared_text": "Gracias a todos por venir",
        },
        {
          "name": "Eric Cartman",
          "email": "stan@cc.com",
          "recipient_identification": "AB-12345",
          "custom_text": "Artísta plastilínico",
          "motive": "Arte con plastilina",
          "date": "3 de marzo de 1999",
          "place": "Sout Park",
          "shared_text": "Gracias a todos por venir",
        },
      ]))?,
    };

    let mut issuance = w.process().await?;

    let entries = issuance.entry_vec().await?;
    assert_eq!(entries.len(), 2);
    assert_that!(&entries, all_elements_satisfy(|e: &Entry| e.flow().is_received()));

    site.request().create_all_received().await?;

    assert_eq!(
      &issuance.entry_vec().await?[0].params_and_custom_message().await?.1.unwrap(),
      "Hola Stan Marsh, esta es una invitación para el evento llamado Arte con plastilina."
    );

    issuance.reload().await?;
    assert!(issuance.flow().is_created());
    assert_that!(&issuance.entry_vec().await?, all_elements_satisfy(|e: &Entry| e.flow().is_created()));

    issuance.flow().for_adding_entries().await?.unwrap()
      .append_entries(&serde_json::from_value::<Vec<HashMap<String,String>>>(serde_json::json!([
        {
          "name": "Kenny Mcormic",
          "email": "kenny@cc.com",
          "recipient_identification": "AB-12345",
          "custom_text": "Artísta plastilínico",
          "motive": "Arte con plastilina",
          "date": "3 de marzo de 1999",
          "place": "Sout Park",
          "shared_text": "Gracias a todos por venir",
        },
        {
          "name": "Kyle Broflovsky",
          "email": "kyle@cc.com",
          "recipient_identification": "AB-12345",
          "custom_text": "Artísta plastilínico",
          "motive": "Arte con plastilina",
          "date": "3 de marzo de 1999",
          "place": "Sout Park",
          "shared_text": "Gracias a todos por venir",
        },
      ]))?).await?;

    issuance.reload().await?;
    assert!(issuance.flow().is_received());

    site.request().create_all_received().await?;
    issuance.reload().await?;
    assert!(issuance.flow().is_created());
    assert_eq!(issuance.entry_scope().count().await?, 4);
    assert_that!(&issuance.entry_vec().await?, all_elements_satisfy(|e: &Entry| e.flow().is_created()));
    assert_eq!(issuance.entry_vec().await?[3].attrs.row_number, 4);
  }

  dbtest!{ fails_to_build_from_json_with_wrong_fields(_site, c)
    let a = c.alice().await;
    let person = a.person().await;

    let w = JsonIssuanceBuilder {
      person,
      name: "Some diploma 2023".to_string(),
      template: WizardTemplate::New {
        name: "A diploma template".to_string(),
        logo: ImageOrText::Text("SuperUni".to_string()),
        kind: TemplateKind::Invitation,
      },
      entries: serde_json::from_value(serde_json::json!([{ "ñombré": "Stan Marsh" }]))?,
    };

    w.process().await.unwrap_err();
  }

  test!{ creates_translated_templates
    assert_eq!(
      &WizardTemplate::make_template_zip(i18n::Lang::Es, ImageOrText::Text("test".to_string()), TemplateKind::Diploma).await?.0,
      "Hola {{ name }}, este es tu diploma de {{ motive }}."
    );

    assert_eq!(
      &WizardTemplate::make_template_zip(i18n::Lang::En, ImageOrText::Text("test".to_string()), TemplateKind::Invitation).await?.0,
      "Hello {{ name }}, this is an invitation for you to attend {{ motive }}."
    );
  }

  dbtest!{ submits_wizard_using_issuer_name(site, c)
    let a = c.alice().await;
    let person = a.person().await;

    let w = Wizard{
      person,
      name: "Some diploma 2023".to_string(),
      template: WizardTemplate::New {
        name: "A diploma template".to_string(),
        logo: ImageOrText::Text("City Wok".to_string()),
        kind: TemplateKind::Attendance,
      },
      csv: read("wizard/default.csv"),
    };

    let request = w.process().await?;
    site.request().create_all_received().await?;
    assert_eq!(request.entry_scope().count().await?, 2);
    std::fs::write("../target/artifacts/zip_from_submits_wizard_using_issuer_name_entry.zip", &request.entry_scope().one().await?.payload().await?)?;

    let mut zipfile = zip::ZipArchive::new(std::io::Cursor::new(request.template().await?.payload().await?))?;
    let mut inner = zipfile.by_index(0)?;
    assert_eq!(inner.name(), "attendance.html.tera");
    let mut contents = String::new();
    inner.read_to_string(&mut contents)?;
    assert_that!(&contents, rematch(r#"\{\{ name \}\}"#));
    assert_that!(&contents, rematch("City Wok"));

    std::fs::write("../target/artifacts/template_from_submits_wizard_using_issuer_name.html", &contents)?;
  }

  dbtest!{ submits_wizard_using_unrecognized_image(_site, c)
    let a = c.alice().await;
    let person = a.person().await;

    let w = Wizard{
      person,
      name: "Some diploma 2023".to_string(),
      template: WizardTemplate::New {
        name: "A diploma template".to_string(),
        logo: ImageOrText::Image(read("wizard/default.csv")),
        kind: TemplateKind::Invitation,
      },
      csv: read("wizard/default.csv"),
    };

    assert_that!(
      &w.process().await.unwrap_err(),
      structure![Error::Validation { field: rematch("logo_image"), message: rematch("not_a_valid_image_file") }]
    );
  }

  dbtest!{ submits_wizard_with_existing_template(site, c)
    let a = c.alice().await;
    let template = a.make_template(
      read("certos_template.zip"),
    ).await;
    let person = a.person().await;

    let w = Wizard{
      person,
      name: "Some diploma 2023".to_string(),
      template: WizardTemplate::Existing {
        template_id: template.attrs.id,
      },
      csv: read("certos_request.csv"),
    };

    let request = w.process().await?;
    site.request().create_all_received().await?;
    assert_eq!(request.entry_scope().count().await?, 2);

    let bob = c.bob().await.person().await;
    let failing_wizard = Wizard{
      person: bob,
      name: "Bob's trying to use alice's template".to_string(),
      template: WizardTemplate::Existing {
        template_id: template.attrs.id,
      },
      csv: read("certos_request.csv"),
    };

    assert_that!(
      &failing_wizard.process().await.unwrap_err(),
      structure![Error::DatabaseError [is_variant!(sqlx::Error::RowNotFound)] ]
   );
  }

  dbtest!{ accepts_csv_with_semicolon (site, c)
    let alice = c.alice().await;
    let request = set_up_csv_request_with_custom_template(
      &alice,
      "src/test_support/resources/certos_request_semicolon.csv"
    ).await?;
    site.request().create_all_received().await?;
    let entries = request.entry_vec().await?;
    assert_eq!(2, entries.len());
    assert_that!(entries[0].params(), rematch("\"curso\":\"Derecho\""));
  }

  regtest!{ bad_certos_request_unequal_lengths (_site, c, _chain)
    set_up_csv_request_with_custom_template(
      &c.alice().await,
      "src/test_support/resources/bad_certos_request_unequal_lengths.csv"
    ).await.unwrap_err();
  }

  regtest!{ bad_certos_request_non_ascii_character (_site, c, _chain)
    set_up_csv_request_with_custom_template(
      &c.alice().await,
      "src/test_support/resources/bad_certos_request_non_ascii_character.csv"
    ).await.unwrap_err();
  }

  regtest!{ bad_certos_request_incompatible_with_template (site, c, _chain)
    let mut request = set_up_csv_request_with_custom_template(
      &c.alice().await,
      "src/test_support/resources/bad_certos_request_incompatible_with_template.csv"
    ).await?;
    
    site.request().create_all_received().await.unwrap_err(); // Ahora se crean todos los documentos.

    request.reload().await?;
    assert!(request.is_failed());
    assert_that!(&request.entry_vec().await?, all_elements_satisfy(|a: &Entry| a.flow().is_failed() ));
  }

  dbtest!{ accepts_crlf_requests (site, c)
    let request = set_up_csv_request_with_custom_template(
      &c.alice().await,
      "src/test_support/resources/certos_request_crlf.csv"
    ).await.expect("Valid request");
    site.request().create_all_received().await?;
    assert_that!(&request.entry_vec().await?[1].attrs.params, rematch("Matemáticas"));
  }

  regtest!{ user_discards_request (site, c, _chain)
    let request = set_up_csv_request_with_custom_template(
      &c.alice().await,
      "src/test_support/resources/certos_request.csv"
    ).await?;
    
    assert_that!(&request.entry_vec().await?, all_elements_satisfy(|a: &Entry| a.flow().is_received() ));

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
    alice.make_request(*template.id(), request_file).await?;

    site.request().create_all_received().await?;
    let templates_files = Template::read_name_and_bytes_from_payload(&template.storage_fetch().await?).await?;
    assert_eq!(templates_files.len(), 3)
  }

  async fn set_up_csv_request_with_custom_template(alice: &SignerClient, request_path: &str) -> ConstataResult<Request> {
    let template_file = read("certos_template.zip");
    let template = alice.make_template(template_file).await;
    let request_file = std::fs::read(request_path)?;
    alice.make_request(*template.id() ,request_file).await
  }
}
