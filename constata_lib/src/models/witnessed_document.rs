use crate::{
  models::{Story, Document, PersonId, document::MimeOverride, model, DocumentSource},
  Site, Result,
};

model!{
  state: Site,
  table: witnessed_documents,
  struct WitnessedDocument {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(varchar)]
    document_id: String,
    evidence: Vec<u8>,
  }
}

impl WitnessedDocumentHub {
  pub async fn create_from_payload
  (&self, story: &Story, payload: &[u8], evidence: &[u8], author_id: PersonId, mime: MimeOverride, sourced_from: DocumentSource, always_gift: bool)
   -> Result<Document>{

    let doc = self.state.document().create_and_index(&story, payload, None, author_id, mime, sourced_from, always_gift).await?;

    self.state.witnessed_document()
      .insert(InsertWitnessedDocument{
        document_id: doc.attrs.id.clone(),
        evidence: evidence.to_vec(),
      })
      .save().await?;

    Ok(doc)
  }
}

describe! {
  use crate::models::document::DocumentAttrs;
  use crate::models::document_part::DocumentPartAttrs;

  dbtest!{ creates_a_document_from_a_witnessed_email (_db, c)
    let bot = c.bot().await.accept_terms_and_conditions().await;
    let document = bot.witnessed_email_with_story().await;

    assert_that!(&document.attrs, structure!(DocumentAttrs {
      id: rematch("1-997d5e04f7f36975ac27224139d23bac36e9da9f23a6d7c9ea9cd1bdb2dca2b5-[a-f0-9]{16}"),
      person_id: eq(1),
    }));
    assert_eq!(document.attachments_count().await?, 6);
    assert!(document.is_accepted());
  }

  dbtest!{ creates_a_document_from_a_nested_multipart_email_forcing_mime_type (_db, c)
    let bot = c.bot().await.accept_terms_and_conditions().await;
    let document = bot.witnessed_email_with_story_and_payload(
      &read("realistic_email"),
      Some(("message/rfc822".to_string(), "email".to_string())),
    ).await;

    assert_that!(&document.attrs, structure!(DocumentAttrs {
      id: rematch("1-c2f8020ce6272494011fd9a73ebffa0785619b31a7d2fc2787cbbf347bbb4d1d-[a-f0-9]{16}"),
      person_id: eq(1),
    }));

    let parts = document.document_part_vec().await?;

    assert_document_part!(
      &parts[0],
      0,
      true,
      "1-c2f8020ce6272494011fd9a73ebffa0785619b31a7d2fc2787cbbf347bbb4d1d",
      "Security alert",
      "c2f8020ce6272494011fd9a73ebffa0785619b31a7d2fc2787cbbf347bbb4d1d",
      "message/rfc822",
      6759
    );

    assert_document_part!(
      &parts[1],
      0,
      false,
      "1-c2f8020ce6272494011fd9a73ebffa0785619b31a7d2fc2787cbbf347bbb4d1d",
      "unnamed_attachment.txt",
      "dd098c1a761348835cb8c8fbec57af5833485a1f0f734756e6d176980a4d894b",
      "text/plain",
      673
    );

    assert_document_part!(
      &parts[2],
      0,
      false,
      "1-c2f8020ce6272494011fd9a73ebffa0785619b31a7d2fc2787cbbf347bbb4d1d",
      "unnamed_attachment.txt",
      "e032a124fecb5811110dbcf774d7ea37ffde85a441ababe96cc1ddda70e8f1a3",
      "text/plain",
      77
    );

    assert_document_part!(
      &parts[3],
      0,
      false,
      "1-c2f8020ce6272494011fd9a73ebffa0785619b31a7d2fc2787cbbf347bbb4d1d",
      "unnamed_attachment.txt",
      "33d9f7698cff20de5c9db8b96162821caf234c10d12d25073dbeda1322b64c98",
      "text/html",
      146
    );

    assert_eq!(document.attachments_count().await?, 4);
    assert!(document.is_accepted());
  }

  dbtest!{ creates_a_document_from_a_single_text_part_email (_db, c)
    let bot = c.bot().await.accept_terms_and_conditions().await;
    let document = bot.witnessed_email_with_story_and_payload(
      &read("single_part_email"),
      Some(("message/rfc822".to_string(), "email".to_string())),
    ).await;

    assert_that!(&document.attrs, structure!(DocumentAttrs {
      id: rematch("1-b1edfed06971bba54a1d79d6946968819e2051f3834fb3764420b4fd05199d3e-[a-f0-9]{16}"),
      person_id: eq(1),
    }));

    let parts = document.document_part_vec().await?;
    assert_document_part!(
      &parts[0],
      0,
      true,
      "1-b1edfed06971bba54a1d79d6946968819e2051f3834fb3764420b4fd05199d3e",
      "PEDRITO LLC - Constata / Verificación avanzada de identidad ",
      "b1edfed06971bba54a1d79d6946968819e2051f3834fb3764420b4fd05199d3e",
      "message/rfc822",
      5297
    );
    
    assert_document_part!(
      &parts[1],
      0,
      false,
      "1-b1edfed06971bba54a1d79d6946968819e2051f3834fb3764420b4fd05199d3e",
      "PEDRITO LLC - Constata / Verificación avanzada de identidad '",
      "7e2ff35c7ca49fffbf14815d4858967cc7b3998965dfb259667cc4f96ffc8b68",
      "text/plain",
      112
    );

    assert_eq!(document.attachments_count().await?, 2);
  }
}
