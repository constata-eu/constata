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
      "1390c7f9bb6fae0306feaecbbdf8683ff870fe882b25aeb100ddc043a65c9d13",
      "text/plain",
      79
    );

    assert_document_part!(
      &parts[3],
      0,
      false,
      "1-c2f8020ce6272494011fd9a73ebffa0785619b31a7d2fc2787cbbf347bbb4d1d",
      "unnamed_attachment.txt",
      "efa55663c0049c923c063758b08d08f321236661fe4e963d11b4270bbc937979",
      "text/html",
      148
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
      id: rematch("1-052e12431cb9ee2b64c42ff7ce775c4f4dbe166361b08935ee3c0359cf7da920-[a-f0-9]{16}"),
      person_id: eq(1),
    }));

    let parts = document.document_part_vec().await?;
    assert_document_part!(
      &parts[0],
      0,
      true,
      "1-052e12431cb9ee2b64c42ff7ce775c4f4dbe166361b08935ee3c0359cf7da920",
      "PEDRITO LLC - Constata / Verificaci??n avanzada de identidad ",
      "052e12431cb9ee2b64c42ff7ce775c4f4dbe166361b08935ee3c0359cf7da920",
      "message/rfc822",
      5297
    );
    
    assert_document_part!(
      &parts[1],
      0,
      false,
      "1-052e12431cb9ee2b64c42ff7ce775c4f4dbe166361b08935ee3c0359cf7da920",
      "PEDRITO LLC - Constata / Verificaci??n avanzada de identidad '",
      "46e415c9fed7fffebc6d868a1bb0585814ddcfaa7663ce3119ee277fce36a640",
      "text/plain",
      114
    );

    assert_eq!(document.attachments_count().await?, 2);
  }
}
