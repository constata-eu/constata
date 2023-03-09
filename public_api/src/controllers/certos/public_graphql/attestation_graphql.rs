use super::*;
use constata_lib::signed_payload;
use serde::{Deserialize, Serialize};
use rust_decimal_macros::dec;

#[derive(GraphQLInputObject, Serialize, Deserialize)]
#[graphql(description = "An AttestationInput has all parameters required to create an Attestation on several of documents.")]
#[serde(rename_all = "camelCase")]
pub struct AttestationInput {
  #[graphql(description = "An array of SignedPayloads containing all the documents to attest. See the tutorial for more info on signing payloads.")]
  documents: Vec<SignedPayload>,
  #[graphql(description = "An attestation allows appending documents up until a certain date. If you don't chose a date, no appending will be allowed.")]
  open_until: Option<UtcDateTime>,
  #[graphql(description = "Markers is a text that can be used for searching this attestation later. Markers cannot be updated after creation.")]
  markers: Option<String>,
  #[graphql(description = "A list of email addresses to notify when the documents are attested. Constata will email them an administrative access link to view, download or share the document certificate. You can pass an empty list if you want to omit Constata's emails, and manage distribution of the attestation in any other way.")]
  email_admin_access_url_to: Vec<String>,
}

impl AttestationInput {
  pub async fn create_attestation(self, context: &Context) -> FieldResult<Attestation> {
    let person = context.person();
    let payloads = self.documents.iter()
      .map(|d| d.decode() )
      .collect::<MyResult<Vec<signed_payload::SignedPayload>>>()?;

    let att = context.site.attestation()
      .create(&person, &payloads, self.open_until, self.markers, Some(context.lang), self.email_admin_access_url_to)
      .await?;

    Ok(Attestation::db_to_graphql(att, false).await?)
  }
}

#[derive(GraphQLInputObject, Debug, PartialEq, Clone, Deserialize, Serialize)]
#[graphql(description = "An AttestationInput has all parameters required to create an Attestation on several of documents.")]
pub struct SignedPayload {
  pub payload: String,
  pub signer: String,
  pub signature: String,
}

impl SignedPayload {
  pub fn decode(&self) -> MyResult<signed_payload::SignedPayload> {
    Ok(serde_json::from_str(&serde_json::to_string(self)?)?)
  }
}

#[derive(GraphQLObject)]
#[graphql(description = "An Attestation over several documents")]
pub struct Attestation {
  id: i32,
  person_id: i32,
  org_id: i32,
  markers: String,
  open_until: Option<UtcDateTime>,
  state: String,
  parking_reason: Option<String>,
  done_documents: i32,
  parked_documents: i32,
  processing_documents: i32,
  total_documents: i32,
  tokens_cost: f64,
  tokens_paid: f64,
  tokens_owed: f64,
  buy_tokens_url: Option<String>,
  accept_tyc_url: Option<String>,
  last_doc_date: Option<UtcDateTime>,
  email_admin_access_url_to: Vec<String>,
  admin_access_url: Option<String>,
  created_at: UtcDateTime,
}

#[derive(GraphQLObject)]
#[graphql(description = "You can get an attestation as a verifiable HTML, embedding all documents and verifiable in any default browser.")]
pub struct AttestationHtmlExport {
  pub id: i32,
  pub attestation: Attestation,
  pub verifiable_html: String,
}

#[derive(Clone, GraphQLInputObject, Debug)]
pub struct AttestationFilter {
  ids: Option<Vec<i32>>,
  id_eq: Option<i32>,
  person_id_eq: Option<i32>,
  markers_like: Option<String>,
}

#[rocket::async_trait]
impl Showable<attestation::Attestation, AttestationFilter> for Attestation {
  fn sort_field_to_order_by(field: &str) -> Option<AttestationOrderBy> {
    match field {
      "id" => Some(AttestationOrderBy::Id),
      "personId" => Some(AttestationOrderBy::PersonId),
      "createdAt" => Some(AttestationOrderBy::CreatedAt),
      _ => None,
    }
  }

  fn filter_to_select(org_id: i32, f: AttestationFilter) -> SelectAttestation {
    SelectAttestation {
      id_in: f.ids,
      org_id_eq: Some(org_id),
      id_eq: f.id_eq,
      person_id_eq: f.person_id_eq,
      markers_ilike: into_like_search(f.markers_like),
      deletion_id_is_set: Some(false),
      ..Default::default()
    }
  }

  fn select_by_id(org_id: i32, id: i32) -> SelectAttestation {
    SelectAttestation { id_eq: Some(id), org_id_eq: Some(org_id), ..Default::default() }
  }

  async fn db_to_graphql(d: attestation::Attestation, _with_payload: bool) -> MyResult<Self> {
    let story = d.story().await?;
    let account_state = d.org().await?.account_state().await?;
    
    let mut email_admin_access_url_to = std::collections::HashSet::new();
    let mut tokens_cost = dec!(0);
    let mut tokens_paid = dec!(0);
    let mut tokens_owed = dec!(0);
    let mut done_documents = 0;
    let mut parked_documents = 0;
    let mut processing_documents = 0;

    for doc in &story.documents().await? {
      tokens_cost += doc.attrs.cost;
      if doc.attrs.funded {
        tokens_paid += doc.attrs.cost;
      } else {
        tokens_owed += doc.attrs.cost;
      }
      if doc.bulletin().await?.map(|b| b.is_published()).unwrap_or(false) {
        done_documents += 1;
      } else if doc.is_parked() {
        parked_documents += 1;
      } else {
        processing_documents += 1;
      }
      for cb in doc.email_callback_scope().cc_eq(true).all().await? {
        email_admin_access_url_to.insert(cb.attrs.address);
      }
    }

    let state = if done_documents > 0 {
      if parked_documents == 0 && processing_documents == 0 {
        "done"
      } else if parked_documents > 0 {
        "updates_parked"
      } else {
        "updates_processing"
      }
    } else {
      if parked_documents > 0 {
        "parked"
      } else {
        "processing"
      }
    };

    let parking_reason = if state == "parked" || state == "updates_parked" {
      if account_state.pending_tyc_url.is_some() {
        Some("must_accept_tyc")
      } else {
        Some("must_buy_tokens")
      }
    } else {
      None
    };

    let admin_access_url = story.create_download_proof_link(30).await?;
    let last_doc_date = story.documents().await?.last().map(|d| d.attrs.created_at.clone());

    Ok(Attestation {
      id: d.attrs.id,
      person_id: d.attrs.person_id,
      org_id: d.attrs.org_id,
      state: state.to_string(),
      parking_reason: parking_reason.map(|x| x.to_string()),
      open_until: story.attrs.open_until,
      markers: d.attrs.markers,
      created_at: d.attrs.created_at,
      email_admin_access_url_to: Vec::from_iter(email_admin_access_url_to),
      admin_access_url: admin_access_url,
      buy_tokens_url: account_state.pending_invoice_link_url,
      accept_tyc_url: account_state.pending_tyc_url,
      done_documents,
      parked_documents,
      processing_documents,
      total_documents: done_documents + parked_documents + processing_documents,
      tokens_cost: tokens_cost.to_f64().unwrap_or(0.0),
      tokens_paid: tokens_paid.to_f64().unwrap_or(0.0),
      tokens_owed: tokens_owed.to_f64().unwrap_or(0.0),
      last_doc_date,
    })
  }
}

constata_lib::describe_one! {
  fulltest!{ can_create_an_attestation (_site, c, client, mut chain)
    use chrono::prelude::*;

    use gql::{
      *,
      create_attestation as create,
      attestation as show,
      all_attestations as all,
      attestation_html_export as export,
    };

    client.signer.verify_email("test@example.com").await;

    let vars = create::Variables{
      input: create::AttestationInput {
        documents: vec![
          client.signer.signed_payload(b"hello world").into(),
          client.signer.signed_payload(b"goodbye world").into(),
        ],
        open_until: Some(chrono::Utc.with_ymd_and_hms(2050, 1, 1, 1, 1, 1).unwrap()),
        markers: Some("foo bar baz".to_string()),
        email_admin_access_url_to: vec!["foo@example.com".to_string(), "bar@example.com".to_string()]
      }
    };

    let created: create::ResponseData = client.gql(&CreateAttestation::build_query(vars)).await;

    assert_that!(&created, structure!{ create::ResponseData {
      create_attestation: structure! { create::CreateAttestationCreateAttestation {
        id: eq(1),
        org_id: eq(1),
        markers: rematch("foo bar baz"),
        state: rematch("processing"),
        open_until: maybe_some(eq(chrono::Utc.with_ymd_and_hms(2050, 1, 1, 1, 1, 1).unwrap())),
        parking_reason: eq(None),
        done_documents: eq(0),
        parked_documents: eq(0),
        processing_documents: eq(2),
        total_documents: eq(2),
        tokens_cost: eq(2.0),
        tokens_paid: eq(2.0),
        tokens_owed: eq(0.0),
        buy_tokens_url: eq(None),
        accept_tyc_url: eq(None),
        email_admin_access_url_to: contains_in_any_order(vec!["foo@example.com".to_string(), "bar@example.com".to_string()]),
        admin_access_url: eq(None),
      }}
    }});

    let processing: show::ResponseData = client.gql(&Attestation::build_query(show::Variables{ id: 1 })).await;

    assert_that!(&processing, structure!{ show::ResponseData {
      attestation: structure! { show::AttestationAttestation {
        id: eq(1),
        org_id: eq(1),
        state: rematch("processing"),
        done_documents: eq(0),
        parked_documents: eq(0),
        processing_documents: eq(2),
        total_documents: eq(2),
        admin_access_url: eq(None),
      }}
    }});

    chain.fund_signer_wallet();
    chain.simulate_stamping().await;

    let done: show::ResponseData = client.gql(&Attestation::build_query(show::Variables{ id: 1 })).await;

    assert_that!(&done, structure!{ show::ResponseData {
      attestation: structure! { show::AttestationAttestation {
        id: eq(1),
        org_id: eq(1),
        state: rematch("done"),
        done_documents: eq(2),
        parked_documents: eq(0),
        processing_documents: eq(0),
        total_documents: eq(2),
        admin_access_url: maybe_some(rematch("http://localhost:8000/safe/.*")),
      }}
    }});

    let search = all::Variables{
      page: Some(0),
      sort_field: Some("createdAt".to_string()),
      per_page: None,
      sort_order: None,
      filter: Some(all::AttestationFilter{
        markers_like: Some("foo".to_string()),
        id_eq: None,
        ids: None,
        person_id_eq: None,
      }),
    };
    let attestations: all::ResponseData = client.gql(&AllAttestations::build_query(search)).await;

    assert_that!(&attestations.all_attestations[0], structure!{
      all::AllAttestationsAllAttestations {
        id: eq(1),
      }
    });

    let empty_search = all::Variables{
      page: None,
      sort_field: None,
      per_page: None,
      sort_order: None,
      filter: Some(all::AttestationFilter{
        markers_like: Some("bogus".to_string()),
        id_eq: None,
        ids: None,
        person_id_eq: None,
      }),
    };
    let empty_list: all::ResponseData = client.gql(&AllAttestations::build_query(empty_search)).await;

    assert!(empty_list.all_attestations.is_empty());

    let exported: export::ResponseData = client.gql(&AttestationHtmlExport::build_query(export::Variables{ id: 1 })).await;

    assert_that!(&exported, structure!{ export::ResponseData {
      attestation_html_export: structure! { export::AttestationHtmlExportAttestationHtmlExport {
        id: eq(1),
        verifiable_html: rematch("html"),
        attestation: structure!{ export::AttestationHtmlExportAttestationHtmlExportAttestation {
          id: eq(1),
          org_id: eq(1),
          state: rematch("done"),
          done_documents: eq(2),
          parked_documents: eq(0),
          processing_documents: eq(0),
        }}
      }}
    }});
  }
}

