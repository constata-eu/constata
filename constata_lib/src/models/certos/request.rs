use crate::{
  models::{
    model,
    Site,
    UtcDateTime,
    entry::*,
    template::*,
    person::*,
    OrgDeletion,
    Org,
    Decimal,
    storable::*,
  },
  Error, Result, 
};
use byte_unit::n_mb_bytes;
use csv;
use num_traits::ToPrimitive;
use duplicate::duplicate_item;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use bitcoin::{
  util::misc::MessageSignature,
};

model!{
  state: Site,
  table: certos_requests,
  struct Request {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    app_id: i32,
    #[sqlx_model_hints(int4)]
    person_id: PersonId,
    #[sqlx_model_hints(int4)]
    org_id: i32,
    #[sqlx_model_hints(int4)]
    template_id: i32,
    #[sqlx_model_hints(varchar)]
    state: String,
    #[sqlx_model_hints(varchar)]
    name: String,
    #[sqlx_model_hints(varchar, default)]
    errors: Option<String>,
    #[sqlx_model_hints(timestamptz, default)]
    created_at: UtcDateTime,
    size_in_bytes: i32,
    #[sqlx_model_hints(int4, default)]
    deletion_id: Option<i32>,
  },
  has_many {
    Entry(request_id),
  },
  belongs_to {
    Template(template_id),
    Person(person_id),
    Org(org_id),
    OrgDeletion(deletion_id),
  }
}
derive_storable!(Request, "wr");

impl RequestHub {
  pub async fn create_all_received(&self) -> Result<()> {
    for r in &self.select().state_eq(&"received".to_string()).all().await? {
      r.in_received()?.create().await?;
    }
    Ok(())
  }

  pub async fn try_complete(&self) -> Result<()> {
    for r in &self.select().state_eq(&"signed".to_string()).all().await? {
      r.in_signed()?.try_complete().await?;
    }
    Ok(())
  }
}

impl Request {
  pub async fn payload(&self) -> Result<Vec<u8>> {
    self.storage_fetch().await
  }

  pub fn flow(&self) -> Flow {
    match self.state().as_ref() {
      "received" => Flow::Received(Received(self.clone())),
      "created" => Flow::Created(Created(self.clone())),
      "signed" => Flow::Signed(Signed(self.clone())),
      "completed" => Flow::Completed(Completed(self.clone())),
      _ => Flow::Failed(Failed(self.clone())),
    }
  }

  pub async fn read_csv_from_payload(reader_buffer: &[u8]) -> csv::Reader<&[u8]> {
    let separator = if String::from_utf8_lossy(reader_buffer).contains(",") {
      b','
    } else {
      b';'
    };
    csv::ReaderBuilder::new().delimiter(separator).from_reader(reader_buffer)
  }
}

/*
 *  Flow state:     What should users do:     What's going on:
 *  received        wait                      constata must validate this entry and render the files.
 *  created         review and sign           customer must now review and sign this document.
 *  signed          wait                      customer may need to pay, accept TyC, and wait for constata to timestamp the document.
 *  completed       nothing else              constata has sent this file to the recipients.
 *  failed          fix the issue and retry.  something went wrong during creation, either detected by constata or by the user.
 */
#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(tag = "state")]
pub enum Flow {
  Received(Received),
  Created(Created),
  Signed(Signed),
  Completed(Completed),
  Failed(Failed),
}

#[duplicate_item(flow_variant; [ Received ]; [ Created ]; [ Signed ]; [ Completed ]; [ Failed ];)]
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct flow_variant(Request);

#[duplicate_item(flow_variant; [ Received ]; [ Created ]; [ Signed ]; [ Completed ]; [ Failed ];)]
impl flow_variant {
  pub fn id(&self) -> &i32 { self.0.id() }
  pub fn created_at(&self) -> &UtcDateTime { self.0.created_at() }
  pub fn into_inner(self) -> Request { self.0 }
  pub fn as_inner<'a>(&'a self) -> &'a Request { &self.0 }
  pub async fn entry_vec(&self) -> sqlx::Result<Vec<Entry>> { self.0.entry_vec().await }
}

#[duplicate_item(
  in_state          is_state          state_str       state_struct;
  [ in_received   ] [ is_received   ] [ "received"  ] [ Received  ];
  [ in_created    ] [ is_created    ] [ "created"   ] [ Created   ];
  [ in_signed     ] [ is_signed     ] [ "signed"    ] [ Signed    ];
  [ in_completed  ] [ is_completed  ] [ "completed" ] [ Completed ];
  [ in_failed     ] [ is_failed     ] [ "failed"    ] [ Failed    ];
)]
impl Request {
  pub fn in_state(&self) -> Result<state_struct> {
    self.flow().in_state()
  }

  pub fn is_state(&self) -> bool {
    self.attrs.state.as_str() == state_str
  }
}

#[duplicate_item(
  in_state          is_state          variant(i)            state_struct;
  [ in_received   ] [ is_received   ] [ Flow::Received(i) ] [ Received  ];
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
  pub fn as_inner<'a>(&'a self) -> &'a Request {
    match self {
      Flow::Received(a) => a.as_inner(),
      Flow::Created(a) => a.as_inner(),
      Flow::Signed(a) => a.as_inner(),
      Flow::Completed(a) => a.as_inner(),
      Flow::Failed(a) => a.as_inner(),
    }
  }
}

impl Received {
  pub async fn create(&self) -> Result<Created> {
    match self.create_helper().await {
      Ok(created) => Ok(created),
      Err(e) => {
        self.to_owned().into_inner().update()
          .state("failed".to_string())
          .errors(Some(e.to_string()))
          .save().await?;
        Err(e)
      }
    }
  }

  pub async fn create_helper(&self) -> Result<Created> {
    let inner = self.as_inner();

    let template_payload = inner.template().await?.payload().await?;
    let template_files = Template::read_name_and_bytes_from_payload(&template_payload).await?;

    let reader_buffer = inner.storage_fetch().await?;
    let mut rows = Request::read_csv_from_payload(&reader_buffer).await;

    for (i, row) in rows.deserialize().enumerate() {
      inner.state.entry().create(inner, i as i32, &template_files, row?).await?;
    }

    inner.to_owned().update().state("created".to_string()).save().await?.in_created()
  }
}

#[serde_as]
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct EntrySignature {
  pub entry_id: i32,
  #[serde_as(as = "DisplayFromStr")]
  pub signature: MessageSignature,
}

impl EntrySignature {
  pub fn from_base64(id: i32, signature_string: &str) -> Result<Self> {
    Ok(Self{
      entry_id: id,
      signature: MessageSignature::from_base64(signature_string)?,
    })
  }
}

impl Created {
  pub async fn tokens_needed(&self) -> Result<i32> {
    let one_mb = Decimal::from(n_mb_bytes!(1));
    let mut tokens = 0;
    
    for entry in self.0.entry_vec().await? {
      if entry.is_created() {
        tokens += (Decimal::from(entry.attrs.size_in_bytes) / one_mb).ceil().to_i32().unwrap_or(0);
      }
    }

    Ok(tokens)
  }

  pub async fn signing_iterator(&self, signature: Option<EntrySignature>) -> Result<Option<Entry>> {
    if let Some(sig) = signature {
      self.as_inner().state.entry()
        .select()
        .request_id_eq(self.id())
        .id_eq(&sig.entry_id)
        .state_eq(&"created".to_string())
        .one().await?
        .in_created()?
        .apply_signature(sig.signature).await?;
    }

    let next = self.as_inner().state.entry()
      .select()
      .request_id_eq(self.id())
      .state_eq(&"created".to_string())
      .optional().await?;

    if next.is_none() {
      self.clone().into_inner().update().state("signed".to_string()).save().await?;
    }

    Ok(next)
  }

  pub async fn discard(&self) -> Result<Failed> {
    self.clone().into_inner().update()
      .state("failed".to_string())
      .errors(Some("user_discarded".to_string()))
      .save().await?.in_failed()
  }
}

impl Signed {
  pub async fn try_complete(&self) -> Result<()> {
    let entries = self.as_inner().state.entry()
      .select()
      .request_id_eq(self.id())
      .state_eq(&"signed".to_string())
      .all().await?;

    let mut all_complete = true;
    for e in entries.into_iter() {
      all_complete = e.in_signed()?.try_complete().await? && all_complete;
    };
    
    if all_complete {
      self.clone().into_inner().update().state("completed".to_string()).save().await?;
    }

    Ok(())
  }
}

impl Failed {
  pub fn errors(&self) -> &str {
    self.as_inner().attrs.errors.as_deref().unwrap_or("")
  }
}

impl InsertRequestHub {
  pub async fn validate_and_save(self, payload: &[u8]) -> Result<Request> {
    let sanitized = if let Err(std::str::Utf8Error{..}) = std::str::from_utf8(&payload) {
      Some(payload.iter().map(|b| *b as char).collect::<String>().into_bytes())
    } else {
      None
    };

    let reader_buffer: &[u8] = &sanitized.as_deref().unwrap_or(payload);

    let mut rows = Request::read_csv_from_payload(reader_buffer).await;

    for header in rows.headers()? {
      if !header.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(Error::validation("payload", "non_ascii_character"));
      }
    }
    
    for result in rows.records() {
      if let Err(error) = result {
        let err = match error.kind() {
          csv::ErrorKind::UnequalLengths{..} => "unequal_lengths",
          csv::ErrorKind::Utf8 {..} => "utf8",
          _ => "unexpected",
        };
        return Err(Error::validation("payload", err));
      }
    }

    let request = self.save().await?;
    request.storage_put(&reader_buffer).await?;
    Ok(request)
  }
}
