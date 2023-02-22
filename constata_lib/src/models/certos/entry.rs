use crate::{
  models::{
    model,
    Site,
    UtcDateTime,
    document::*,
    person::*,
    OrgDeletion,
    certos::{request::*, app::*, template::*, template_kind::TemplateKind},
    email_callback::*,
    Org,
    storable::*,
  },
  signed_payload::SignedPayload,
  Error, Result,
};
use std::collections::HashMap;
use duplicate::duplicate_item;
use serde::Serialize;

use bitcoin::{
  util::misc::MessageSignature,
};

model!{
  state: Site,
  table: certos_entries,
  struct Entry {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    app_id: i32,
    #[sqlx_model_hints(int4)]
    person_id: PersonId,
    #[sqlx_model_hints(int4)]
    org_id: i32,
    #[sqlx_model_hints(int4)]
    request_id: i32,
    #[sqlx_model_hints(int4)]
    row_number: i32,
    #[sqlx_model_hints(varchar)]
    state: String,
    #[sqlx_model_hints(timestamptz, default)]
    created_at: UtcDateTime,
    size_in_bytes: i32,
    #[sqlx_model_hints(varchar)]
    params: String,
    #[sqlx_model_hints(text)]
    errors: Option<String>,
    #[sqlx_model_hints(varchar)]
    document_id: Option<String>,
    #[sqlx_model_hints(int4)]
    email_callback_id: Option<i32>,
    #[sqlx_model_hints(int4, default)]
    deletion_id: Option<i32>,
  },
  belongs_to {
    App(app_id),
    Person(person_id),
    Org(org_id),
    Document(document_id),
    EmailCallback(email_callback_id),
    Request(request_id),
    OrgDeletion(deletion_id),
  }
}

derive_storable!(Entry, "we");

impl Entry {
  pub fn flow(&self) -> Flow {
    match self.state().as_ref() {
      "created" => Flow::Created(Created(self.clone())),
      "signed" => Flow::Signed(Signed(self.clone())),
      "completed" => Flow::Completed(Completed(self.clone())),
      _ => Flow::Failed(Failed(self.clone())),
    }
  }

  pub async fn template_kind(&self) -> Result<TemplateKind> {
    Ok(self.request().await?.template().await?.attrs.kind)
  }

  pub async fn title(&self) -> Result<Option<String>> {
    if let Some(title_override) = self.request().await?.template().await?.attrs.og_title_override {
      return Ok(Some(title_override));
    }
    
    Ok(serde_json::from_str::<HashMap<String, String>>(self.params())?
      .get("motive")
      .map(|motive| motive.to_string()))
  }

  pub async fn params_and_custom_message(&self) -> Result<(HashMap<String, String>, Option<String>)> {
    let params = self.parsed_params()?;
    let custom_message = self.request().await?.template().await?.attrs.custom_message
      .map(|msg| i18n::Tera::one_off(&msg, &i18n::Context::from_serialize(&params)?, true) )
      .transpose()?;

    Ok((params, custom_message))
  }

  pub fn parsed_params(&self) -> Result<HashMap<String,String>> {
    Ok(serde_json::from_str(self.params())?)
  }

  pub async fn payload(&self) -> Result<Vec<u8>> {
    self.storage_fetch().await
  }

  pub async fn notification_status(&self) -> Result<&str> {
    if self.parsed_params()?.get("email").map(|x| x.is_empty()).unwrap_or(true) {
      return Ok("will_not_notify");
    }

    if self.email_callback().await?.map(|cb| cb.sent_at().is_some()).unwrap_or(false) {
      return Ok("notified");
    }

    Ok("will_notify")
  }

  pub async fn admin_access_url(&self) -> Result<Option<String>> {
    if let Some(doc) = self.document().await? {
      if let Some(link) = doc.active_download_proof_link().await? {
        return link.safe_env_url().await.map(|v| Some(v));
      }
     }

    Ok(None)
  }
}

/*
 * A request's state is the same of all of its entries.
 * So look there for the meaning of each state.
 */
#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(tag = "state")]
pub enum Flow {
  Created(Created),
  Signed(Signed),
  Completed(Completed),
  Failed(Failed),
}

#[duplicate_item(flow_variant; [ Created ]; [ Signed ]; [ Completed ]; [ Failed ];)]
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct flow_variant(Entry);

#[duplicate_item(flow_variant; [ Created ]; [ Signed ]; [ Completed ]; [ Failed ];)]
impl flow_variant {
  pub fn id(&self) -> &i32 { self.0.id() }
  pub fn created_at(&self) -> &UtcDateTime { self.0.created_at() }
  pub fn into_inner(self) -> Entry { self.0 }
  pub fn as_inner<'a>(&'a self) -> &'a Entry { &self.0 }
}

#[duplicate_item(
  in_state          is_state          state_str       state_struct;
  [ in_created    ] [ is_created    ] [ "created"   ] [ Created   ];
  [ in_signed     ] [ is_signed     ] [ "signed"    ] [ Signed    ];
  [ in_completed  ] [ is_completed  ] [ "completed" ] [ Completed ];
  [ in_failed     ] [ is_failed     ] [ "failed"    ] [ Failed    ];
)]
impl Entry {
  pub fn in_state(&self) -> Result<state_struct> {
    self.flow().in_state()
  }

  pub fn is_state(&self) -> bool {
    self.attrs.state.as_str() == state_str
  }
}

#[duplicate_item(
  in_state          is_state          variant(i)            state_struct;
  [ in_created    ] [ is_created    ] [ Flow::Created(i)  ] [ Created   ];
  [ in_signed     ] [ is_signed     ] [ Flow::Signed(i)   ] [ Signed    ];
  [ in_completed  ] [ is_completed  ] [ Flow::Completed(i)] [ Completed ];
  [ in_failed     ] [ is_failed     ] [ Flow::Failed(i)   ] [ Failed    ];
)]
impl Flow {
  pub fn in_state(&self) -> Result<state_struct> {
    if let variant([inner]) = self {
      Ok(inner.clone())
    } else {
      Err(Error::InvalidFlowState)
    }
  }

  pub fn is_state(&self) -> bool { matches!(self, variant([_])) }
}

impl Flow {
  pub fn as_inner<'a>(&'a self) -> &'a Entry {
    match self {
      Flow::Created(a) => a.as_inner(),
      Flow::Signed(a) => a.as_inner(),
      Flow::Completed(a) => a.as_inner(),
      Flow::Failed(a) => a.as_inner(),
    }
  }
}

impl EntryHub {
  pub async fn create(self, request: &Request, row_number: i32, template_files: &[(String, Vec<u8>)], row: HashMap<String,String>) -> Result<Created> {
    use std::io::Write;
    use zip::write::FileOptions;

    let params = serde_json::to_string(&row)?;
    let context = i18n::tera::Context::from_serialize(row)?;

    let mut destination_buffer = vec![];

    {
      let mut destination = zip::ZipWriter::new(std::io::Cursor::new(&mut destination_buffer));
      for (raw_name, bytes) in template_files {
        let name = raw_name.split('/').last().unwrap_or(raw_name);
        let (final_name, final_bytes) = if let Some(tera_name) = Template::is_tera(name) {
          let expanded = i18n::Tera::one_off(std::str::from_utf8(bytes)?, &context, true)?.into_bytes();
          (tera_name.to_owned(), expanded)
        } else {
          (name.to_owned(), bytes.to_owned())
        };
        destination.start_file(final_name, FileOptions::default())?;
        destination.write_all(&final_bytes)?;
        destination.flush()?;
      }

      destination.finish()?;
    }

    let entry = self.state.entry().insert(InsertEntry{
      app_id: *request.app_id(),
      person_id: *request.person_id(),
      org_id: *request.org_id(),
      request_id: *request.id(),
      row_number,
      state: "created".to_string(),
      size_in_bytes: destination_buffer.len() as i32,
      params,
      errors: None,
      document_id: None,
      email_callback_id: None,
    })
    .save().await?;

    entry.storage_put(&destination_buffer).await?;

    entry.in_created()
  }
}

impl Created {
  pub async fn apply_signature(self, signature: MessageSignature) -> Result<Signed> {
    let transactional = self.as_inner().state.entry().transactional().await?;
    let this = transactional.find(self.id()).await?;
    let state = &this.state;
    let person = this.person().await?;

    let (params, custom_message) = self.as_inner().params_and_custom_message().await?;

    let lang = custom_message.as_ref()
      .map(|m| i18n::Lang::detect_from_text(&m) )
      .unwrap_or_else(|| person.attrs.lang );

    let story = state.story().create(person.attrs.org_id, None, format!("cert {}", signature), lang).await?;
    let signer = person.pubkey().await?
      .ok_or_else(|| Error::validation("person", "has no pubkey registered. Impossible."))?
      .address();

    let signed_payload = SignedPayload {
      payload: this.payload().await?,
      signer,
      signature
    };

    let document = state.document().create_from_signed_payload(&story, &signed_payload, None).await?;

    let email_callback_id = match params.get("email") {
      Some(address) if address != "" => {
        if person.can_send_email().await? {
          Some(state.email_callback().insert(InsertEmailCallback{
            address: address.to_owned(),
            cc: true,
            document_id: document.attrs.id.clone(),
            custom_message,
            sent_at: None,
          }).validate_and_save().await?.attrs.id)
        } else {
          None
        }
      }
      _ => None,
    };

    let signed = this.update()
      .state("signed".to_string())
      .document_id(Some(document.attrs.id))
      .email_callback_id(email_callback_id)
      .save().await?
      .in_signed();

    transactional.commit().await?;

    signed
  }
}

#[duplicate_item(flow_variant; [ Signed ]; [ Completed ];)]
impl flow_variant {
  pub async fn document(&self) -> sqlx::Result<Document> {
    let inner = self.as_inner();
    inner.state.document()
      .find(&inner.document_id().to_owned().expect("Always has story")).await
  }
}

impl Signed {
  pub async fn try_complete(self) -> Result<bool> {
    let doc = self.document().await?;

    doc.get_or_create_download_proof_link(30).await?;

    let is_published = if let Ok(accepted) = doc.in_accepted() {
      accepted.bulletin().await?.is_published()
    } else {
      false
    };

    let pending_notification = match self.as_inner().email_callback_id() {
      Some(id) => self.as_inner().state.email_callback().find(id).await?.sent_at().is_none(),
      _ => false
    };

    if !is_published || pending_notification {
      return Ok(false)
    }

    self.into_inner().update().state("completed".to_string()).save().await?;
    Ok(true)
  }
}
