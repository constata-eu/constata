use super::*;
use byte_unit::n_mb_bytes;
use csv;
use num_traits::ToPrimitive;
use duplicate::duplicate_item;
use bitcoin::util::misc::MessageSignature;
use std::collections::HashMap;

model!{
  state: Site,
  table: issuances,
  struct Issuance {
    #[sqlx_model_hints(int4, default)]
    id: i32,
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
    #[sqlx_model_hints(int4, default)]
    deletion_id: Option<i32>,
  },
  has_many {
    Entry(issuance_id),
  },
  belongs_to {
    Template(template_id),
    Person(person_id),
    Org(org_id),
    OrgDeletion(deletion_id),
  }
}
derive_storable!(Issuance, "wr");

impl IssuanceHub {
  pub async fn create_all_received(&self) -> ConstataResult<()> {
    for r in &self.select().state_eq(&"received".to_string()).all().await? {
      r.in_received()?.create().await?;
    }
    Ok(())
  }

  pub async fn try_complete(&self) -> ConstataResult<()> {
    for r in &self.select().state_eq(&"signed".to_string()).all().await? {
      r.in_signed()?.try_complete().await?;
    }
    Ok(())
  }
}

impl Issuance {
  pub async fn payload(&self) -> ConstataResult<Vec<u8>> {
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

  pub async fn set_all_failed(self, reason: &str) -> ConstataResult<Failed> {
    let updated = self.update()
      .state("failed".to_string())
      .errors(Some(reason.to_string()))
      .save().await?;

    for e in updated.entry_vec().await? {
      e.update().state("failed".to_string()).save().await?;
    }
    
    updated.in_failed()
  }

  pub async fn export_csv(&self) -> ConstataResult<String> {
    use csv::Writer;

    let mut wtr = Writer::from_writer(vec![]);
    let schema = self.template().await?.parsed_schema()?;

    let mut headers = vec![
      "constata_state",
      "constata_notification_status",
      "constata_admin_access_url",
      "constata_issuance_id",
      "constata_id",
    ];
    headers.extend(schema.iter().map(|i| i.name.as_str() ));

    wtr.write_record(&headers)?;

    for entry in self.entry_scope().order_by(EntryOrderBy::RowNumber).all().await? {
      let mut row = vec![
        entry.state().to_string(),
        entry.notification_status().await?.to_string(),
        entry.admin_access_url().await?.unwrap_or("".to_string()),
        entry.attrs.issuance_id.to_string(),
        entry.attrs.id.to_string()
      ];

      let mut params: HashMap<String,String> = entry.parsed_params()?;
      for attr in &schema {
        row.push(params.remove(&attr.name).unwrap_or("".to_string()));
      }

      wtr.write_record(&row)?;
    }

    Ok(String::from_utf8(wtr.into_inner()?)?)
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
pub struct flow_variant(Issuance);

#[duplicate_item(flow_variant; [ Received ]; [ Created ]; [ Signed ]; [ Completed ]; [ Failed ];)]
impl flow_variant {
  pub fn id(&self) -> &i32 { self.0.id() }
  pub fn created_at(&self) -> &UtcDateTime { self.0.created_at() }
  pub fn into_inner(self) -> Issuance { self.0 }
  pub fn as_inner<'a>(&'a self) -> &'a Issuance { &self.0 }
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
impl Issuance {
  pub fn in_state(&self) -> ConstataResult<state_struct> {
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
  pub fn in_state(&self) -> ConstataResult<state_struct> {
    if let variant([inner]) = self {
      Ok(inner.clone())
    } else {
      Err(Error::InvalidFlowState)
    }
  }

  pub fn is_state(&self) -> bool { matches!(self, variant([_])) }
}

impl Flow {
  pub fn as_inner<'a>(&'a self) -> &'a Issuance {
    match self {
      Flow::Received(a) => a.as_inner(),
      Flow::Created(a) => a.as_inner(),
      Flow::Signed(a) => a.as_inner(),
      Flow::Completed(a) => a.as_inner(),
      Flow::Failed(a) => a.as_inner(),
    }
  }

  pub async fn for_adding_entries(&self) -> ConstataResult<Option<Received>> {
    Ok(match self.to_owned() {
      Flow::Received(a) => Some(a),
      Flow::Created(a) => Some(a.back_to_received().await?),
      _ => None
    })
  }
}

impl Received {
  pub async fn append_entries(&self, rows: &[HashMap<String,String>]) -> ConstataResult<Vec<entry::Received>> {
    let inner = self.as_inner();
    let template_schema = inner.template().await?.parsed_schema()?;

    let base_index = inner.entry_scope().count().await? as i32;
    let mut received = vec![];
    for (i, p) in rows.iter().enumerate() {
      if let Some(field) = Self::maybe_missing_field(&template_schema, p) {
        return Err(Error::validation( field, &format!("Entry #{} is missing field {}. See your selected template's schema for more information.", i, field)));
      }

      received.push(inner.state.entry().insert(InsertEntry{
        person_id: *inner.person_id(),
        org_id: *inner.org_id(),
        issuance_id: *inner.id(),
        row_number: 1 + base_index + i as i32,
        state: "received".to_string(),
        params: serde_json::to_string(&p)?,
      }).save().await?.in_received()?);
    }
    Ok(received)
  }

  pub fn maybe_missing_field<'a>(schema: &'a TemplateSchema, params: &'a HashMap<String,String>) -> Option<&'a str> {
    for field in schema {
      if !field.optional && !params.contains_key(&field.name) {
        return Some(&field.name)
      }
    }
    None
  }

  pub async fn create(&self) -> ConstataResult<Created> {
    match self.create_helper().await {
      Ok(created) => Ok(created),
      Err(e) => {
        self.to_owned().into_inner().set_all_failed("creation_failed").await?;
        Err(e)
      }
    }
  }

  pub async fn create_helper(&self) -> ConstataResult<Created> {
    let inner = self.as_inner();
    let template_payload = inner.template().await?.payload().await?;
    let template_files = Template::read_name_and_bytes_from_payload(&template_payload).await?;

    for entry in inner.entry_scope().state_eq("received".to_string()).all().await? {
      entry.in_received()?.create(&template_files).await?;
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
  pub fn from_base64(id: i32, signature_string: &str) -> ConstataResult<Self> {
    Ok(Self{
      entry_id: id,
      signature: MessageSignature::from_base64(signature_string)?,
    })
  }
}

impl Created {
  pub async fn back_to_received(self) -> ConstataResult<Received> {
    self.into_inner().update().state("received".to_string()).save().await?.in_received()
  }

  pub async fn tokens_needed(&self) -> ConstataResult<i32> {
    let one_mb = Decimal::from(n_mb_bytes!(1));
    let mut tokens = 0;
    
    for entry in self.0.entry_vec().await? {
      if entry.is_created() {
        tokens += (Decimal::from(entry.attrs.size_in_bytes.unwrap_or(0)) / one_mb).ceil().to_i32().unwrap_or(0);
      }
    }

    Ok(tokens)
  }

  pub async fn signing_iterator(&self, signature: Option<EntrySignature>) -> ConstataResult<Option<Entry>> {
    if let Some(sig) = signature {
      self.as_inner().state.entry()
        .select()
        .issuance_id_eq(self.id())
        .id_eq(&sig.entry_id)
        .state_eq(&"created".to_string())
        .one().await?
        .in_created()?
        .apply_signature(sig.signature).await?;
    }

    let next = self.as_inner().state.entry()
      .select()
      .issuance_id_eq(self.id())
      .state_eq(&"created".to_string())
      .optional().await?;

    if next.is_none() {
      self.clone().into_inner().update().state("signed".to_string()).save().await?;
    }

    Ok(next)
  }

  pub async fn discard(&self) -> ConstataResult<Failed> {
    self.to_owned().into_inner().set_all_failed("user_discarded").await
  }
}

impl Signed {
  pub async fn try_complete(&self) -> ConstataResult<()> {
    let entries = self.as_inner().state.entry()
      .select()
      .issuance_id_eq(self.id())
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

describe!{
  use std::io::Read; 
  use std::collections::HashMap;
  use bitcoin::network::constants::Network;
  use crate::models::*;

  regtest!{ process_one_issuance_from_an_arbitrary_template (site, c, mut chain)
    let alice = c.alice().await;
    let email = alice.make_email("alice@example.com").await;
    site.email_address().verify_with_token(&email.access_token().await?.unwrap()).await?;

    let mut issuance = set_up_csv_issuance_with_custom_template(
      &alice,
      "src/test_support/resources/issuance.csv"
    ).await?;
    assert!(issuance.flow().is_received());
    assert_eq!(issuance.entry_scope().state_eq("received".to_string()).count().await?, 2);
    
    site.issuance().create_all_received().await?; // Ahora se crean todos los documentos.
    issuance.reload().await?;

    let export_received = read_to_string("issuance_export_received.csv");
    assert_that!(&issuance.export_csv().await?, rematch(&export_received));

    let created_entries = issuance.entry_vec().await?.into_iter()
      .map(|a| a.in_created() )
      .collect::<ConstataResult<Vec<entry::Created>>>()?;

    assert_eq!(created_entries.len(), 2);

    let payload = created_entries[0].as_inner().payload().await?;
    let mut zipfile = zip::ZipArchive::new(std::io::Cursor::new(&payload))?;

    {
      let inner_a = zipfile.by_index(0).unwrap();
      assert_eq!(inner_a.name(), "2_analítico.html");
    }
    {
      let mut inner_b = zipfile.by_index(1).unwrap();
      assert_eq!(inner_b.name(), "1_diploma.html");
      let mut contents_b = String::new();
      inner_b.read_to_string(&mut contents_b).unwrap();
      assert_that!(&contents_b, rematch("Lisa Simpson"));
    }

    let html_preview = Previewer::create(&payload, true)?.render_html(i18n::Lang::Es)?;
    std::fs::write("../target/artifacts/entry_preview_es_kyc.html", &html_preview)?;
    let html_preview = Previewer::create(&payload, false)?.render_html(i18n::Lang::Es)?;
    std::fs::write("../target/artifacts/entry_preview_es_no_kyc.html", &html_preview)?;

    let html_preview = Previewer::create(&payload, true)?.render_html(i18n::Lang::En)?;
    std::fs::write("../target/artifacts/entry_preview_en_kyc.html", &html_preview)?;
    let html_preview = Previewer::create(&payload, false)?.render_html(i18n::Lang::En)?;
    std::fs::write("../target/artifacts/entry_preview_en_no_kyc.html", &html_preview)?;

    let created = issuance.in_created()?;
    let mut signature = None;
    while let Some(next) = created.signing_iterator(signature).await? {
      signature = Some(alice.sign_issuance_entry(next).await);
    }

    issuance.reload().await?;
    assert!(issuance.is_signed());

    // Now all documents should be accepted.
    for e in &issuance.entry_vec().await? {
      let doc = e.in_signed()?.document().await?;
      assert!(doc.is_accepted());
    }
    assert!(issuance
      .entry_vec().await?[0]
      .in_signed()?
      .document().await?
      .email_callback_vec().await?[0]
      .sent_at().is_none()
    );

    chain.fund_signer_wallet();
    chain.simulate_stamping().await;

    site.issuance().try_complete().await?;
    issuance.reload().await?;
    assert!(!issuance.is_completed());
    assert!(issuance.entry_vec().await?[0].is_signed());
    assert!(issuance.entry_vec().await?[1].is_completed());
   
    let key = TestBlockchain::default_private_key().await?;
    let proof = issuance.entry_vec().await?[1].in_completed()?.document().await?.story().await?.proof(Network::Regtest, &key).await?;
    let content = proof.render_html(i18n::Lang::Es).expect("Content to be ready now");
    std::fs::write("../target/artifacts/diploma_camara.html", &content)?;

    let doc = &issuance.entry_vec().await?[0].in_signed()?.document().await?.in_accepted()?;
    assert!(doc.bulletin().await?.is_published());
    let callback = doc.as_inner().email_callback_vec().await?.pop().unwrap();
    callback.clone().mark_sent().await?;
    let email = callback.render_mailer_html().await?;
    assert_that!(&email, rematch("La empresa Constata.EU le transmite este mensaje"));

    site.issuance().try_complete().await?;
    issuance.reload().await?;
    assert!(issuance.is_completed());
    for e in &issuance.entry_vec().await? {
      assert!(e.is_completed());
    }

    let expected = read_to_string("issuance_export_done.csv");
    assert_that!(&issuance.export_csv().await?, rematch(&expected));
  }

  dbtest!{ submits_wizard_with_new_template(site, c)
    let a = c.alice().await;
    let person = a.person().await;

    let w = Wizard{
      person,
      name: "Some diploma 2023".to_string(),
      template: WizardTemplate::New {
        name: "A badge template".to_string(),
        logo: ImageOrText::Image(read("wizard/logo.png")),
        kind: TemplateKind::Badge,
      },
      csv: read("wizard/default.csv"),
    };

    let issuance = w.process().await?;
    site.issuance().create_all_received().await?;

    assert_eq!(
      &issuance.entry_vec().await?[0].params_and_custom_message().await?.1.unwrap(),
      "Hola Stan Marsh, esta es una insignia por Arte con plastilina."
    );

    assert_eq!(issuance.entry_scope().count().await?, 2);
    std::fs::write("../target/artifacts/entry.zip", &issuance.entry_scope().one().await?.payload().await?)?;


    let mut zipfile = zip::ZipArchive::new(std::io::Cursor::new(issuance.template().await?.payload().await?))?;
    let mut inner = zipfile.by_index(0)?;
    assert_eq!(inner.name(), "badge.html.tera");
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
        kind: TemplateKind::Badge,
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

    site.issuance().create_all_received().await?;

    assert_eq!(
      &issuance.entry_vec().await?[0].params_and_custom_message().await?.1.unwrap(),
      "Hola Stan Marsh, esta es una insignia por Arte con plastilina."
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

    site.issuance().create_all_received().await?;
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
        kind: TemplateKind::Badge,
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
      &WizardTemplate::make_template_zip(i18n::Lang::En, ImageOrText::Text("test".to_string()), TemplateKind::Badge).await?.0,
      "Hello {{ name }}, this is a badge for {{ motive }}."
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

    let issuance = w.process().await?;
    site.issuance().create_all_received().await?;
    assert_eq!(issuance.entry_scope().count().await?, 2);
    std::fs::write("../target/artifacts/zip_from_submits_wizard_using_issuer_name_entry.zip", &issuance.entry_scope().one().await?.payload().await?)?;

    let mut zipfile = zip::ZipArchive::new(std::io::Cursor::new(issuance.template().await?.payload().await?))?;
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
        kind: TemplateKind::Badge,
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
      read("template.zip"),
    ).await;
    let person = a.person().await;

    let w = Wizard{
      person,
      name: "Some diploma 2023".to_string(),
      template: WizardTemplate::Existing {
        template_id: template.attrs.id,
      },
      csv: read("issuance.csv"),
    };

    let issuance = w.process().await?;
    site.issuance().create_all_received().await?;
    assert_eq!(issuance.entry_scope().count().await?, 2);

    let bob = c.bob().await.person().await;
    let failing_wizard = Wizard{
      person: bob,
      name: "Bob's trying to use alice's template".to_string(),
      template: WizardTemplate::Existing {
        template_id: template.attrs.id,
      },
      csv: read("issuance.csv"),
    };

    assert_that!(
      &failing_wizard.process().await.unwrap_err(),
      structure![Error::DatabaseError [is_variant!(sqlx::Error::RowNotFound)] ]
   );
  }

  dbtest!{ accepts_csv_with_semicolon (site, c)
    let alice = c.alice().await;
    let issuance = set_up_csv_issuance_with_custom_template(
      &alice,
      "src/test_support/resources/issuance_semicolon.csv"
    ).await?;
    site.issuance().create_all_received().await?;
    let entries = issuance.entry_vec().await?;
    assert_eq!(2, entries.len());
    assert_that!(entries[0].params(), rematch("\"course\":\"Derecho\""));
  }

  regtest!{ bad_issuance_unequal_lengths (_site, c, _chain)
    set_up_csv_issuance_with_custom_template(
      &c.alice().await,
      "src/test_support/resources/bad_issuance_unequal_lengths.csv"
    ).await.unwrap_err();
  }

  regtest!{ bad_issuance_non_ascii_character (_site, c, _chain)
    set_up_csv_issuance_with_custom_template(
      &c.alice().await,
      "src/test_support/resources/bad_issuance_non_ascii_character.csv"
    ).await.unwrap_err();
  }

  regtest!{ bad_issuance_incompatible_with_template (site, c, _chain)
    use crate::models::TemplateSchemaField;

    let alice = c.alice().await;
    let schema = serde_json::to_string(&vec![
      TemplateSchemaField::new("email", false, false, "Email".into(), "Email".into()),
    ])?;
    let template = alice.try_make_template(read("template.zip"), &schema).await?;
    let mut issuance = alice.make_issuance(
      *template.id(),
      read("bad_issuance_incompatible_with_template.csv")
    ).await?;

    site.issuance().create_all_received().await.unwrap_err(); // Ahora se crean todos los documentos.

    issuance.reload().await?;
    assert!(issuance.is_failed());
    assert_that!(&issuance.entry_vec().await?, all_elements_satisfy(|a: &Entry| a.flow().is_failed() ));
  }

  dbtest!{ accepts_crlf_issuances (site, c)
    let issuance = set_up_csv_issuance_with_custom_template(
      &c.alice().await,
      "src/test_support/resources/issuance_crlf.csv"
    ).await.expect("Valid issuance");
    site.issuance().create_all_received().await?;
    assert_that!(&issuance.entry_vec().await?[1].attrs.params, rematch("Matemáticas"));
  }

  regtest!{ user_discards_issuance (site, c, _chain)
    let issuance = set_up_csv_issuance_with_custom_template(
      &c.alice().await,
      "src/test_support/resources/issuance.csv"
    ).await?;
    
    assert_that!(&issuance.entry_vec().await?, all_elements_satisfy(|a: &Entry| a.flow().is_received() ));

    site.issuance().create_all_received().await?;

    let created = issuance.reloaded().await?.in_created()?;
    let failed = created.discard().await?;
    assert_eq!(failed.errors(), "user_discarded");
  }
  
  dbtest!{ use_template_file_with_image (site, c)
    let alice = c.alice().await;
    let template_file = read("template_with_image.zip");
    let template = alice.make_template(template_file).await;
    let issuance_file = read("issuance.csv");
    alice.make_issuance(*template.id(), issuance_file).await?;

    site.issuance().create_all_received().await?;
    let templates_files = Template::read_name_and_bytes_from_payload(&template.storage_fetch().await?).await?;
    assert_eq!(templates_files.len(), 3)
  }

  async fn set_up_csv_issuance_with_custom_template(alice: &SignerClient, issuance_path: &str) -> ConstataResult<Issuance> {
    let template_file = read("template.zip");
    let template = alice.make_template(template_file).await;
    let issuance_file = std::fs::read(issuance_path)?;
    alice.make_issuance(*template.id() ,issuance_file).await
  }
}
