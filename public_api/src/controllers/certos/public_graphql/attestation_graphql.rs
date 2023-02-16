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
  email_admin_link_to: Vec<String>,
}

impl AttestationInput {
  pub async fn create_attestation(self, context: &Context) -> FieldResult<Attestation> {
    let person = context.person();
    let payloads = self.documents.iter()
      .map(|d| d.decode() )
      .collect::<MyResult<Vec<signed_payload::SignedPayload>>>()?;

    let att = context.site.attestation()
      .create(&person, &payloads, self.open_until, self.markers, Some(context.lang), self.email_admin_link_to)
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
  org_id_eq: Option<i32>,
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
    
    let mut email_admin_access_url_to = vec![];
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
        email_admin_access_url_to.push(cb.attrs.address);
      }
    }

    let state = if done_documents > 0 {
      if parked_documents > 0 {
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
    let last_doc_date = story.pending_docs().await?.last().map(|d| d.attrs.created_at.clone());

    Ok(Attestation {
      id: d.attrs.id,
      person_id: d.attrs.person_id,
      org_id: d.attrs.org_id,
      state: state.to_string(),
      parking_reason: parking_reason.map(|x| x.to_string()),
      open_until: story.attrs.open_until,
      markers: d.attrs.markers,
      created_at: d.attrs.created_at,
      email_admin_access_url_to,
      admin_access_url: admin_access_url,
      buy_tokens_url: account_state.pending_invoice_link_url,
      accept_tyc_url: account_state.pending_tyc_url,
      done_documents,
      parked_documents,
      processing_documents,
      total_documents: done_documents + parked_documents + processing_documents,
      tokens_cost: tokens_cost.to_f64().unwrap_or(0.0),
      tokens_paid: tokens_cost.to_f64().unwrap_or(0.0),
      tokens_owed: tokens_cost.to_f64().unwrap_or(0.0),
      last_doc_date,
    })
  }
}

constata_lib::describe_one! {
  #[derive(Debug, PartialEq, Clone, serde::Deserialize)]
  struct ApiStory { id: i32 }

  fulltest!{ can_create_an_attestation (_site, c, client, mut chain)
    let params = serde_json::json!({
      "documents": vec![
        client.signer.signed_payload(b"hello world"),
        client.signer.signed_payload(b"goodbye world"),
      ],
      "open_until": "2023-03-25T12:00:00Z+0100",
      "markers": "foo bar baz",
      "email_admin_link_to": ["foo@example.com","bar@example.com"]
    });
    /*
     * operationName
     * operationName
     * {"operationName":"createSignup","variables":{"input":{"keepPrivate":false}},"query":"mutation createSignup($input: SignupInput!) {\n  data: createSignup(input: $input) {\n    id\n    __typename\n  }\n}"}
     *
     * Fetching an endorsement manifest:
     * {"operationName":"EndorsementManifest","variables":{"id":1},"query":"query EndorsementManifest($id: Int!) {\n  data: EndorsementManifest(id: $id) {\n    id\n    text\n    websites\n    kyc {\n      name\n      lastName\n      idNumber\n      idType\n      birthdate\n      nationality\n      country\n      jobTitle\n      legalEntityName\n      legalEntityCountry\n      legalEntityRegistration\n      legalEntityTaxId\n      updatedAt\n      __typename\n    }\n    telegram {\n      username\n      firstName\n      lastName\n      __typename\n    }\n    email {\n      address\n      keepPrivate\n      __typename\n    }\n    canSendEmail\n    __typename\n  }\n}"}
     *
     * Creating a Wizard:
     * {"variables":{"input":{"templateId":1,"newKind":"DIPLOMA","name":"Diploma","csv":"name,email,recipient_identification,custom_text,motive,date,place,shared_text\nFoo,,,,Bar,,,\n"}},"query":"mutation ($input: WizardInput!) {\n  data: createWizard(input: $input) {\n    id\n    templateId\n    templateName\n    state\n    name\n    createdAt\n    errors\n    entries\n    __typename\n  }\n}"}
     * 
     * Listing all requests:
     *   {"operationName":"allRequests","variables":{"filter":{},"page":0,"perPage":20,"sortField":"id","sortOrder":"DESC"},"query":"query allRequests($page: Int, $perPage: Int, $sortField: String, $sortOrder: String, $filter: RequestFilter) {\n  items: allRequests(\n    page: $page\n    perPage: $perPage\n    sortField: $sortField\n    sortOrder: $sortOrder\n    filter: $filter\n  ) {\n    id\n    templateId\n    templateName\n    templateKind\n    state\n    name\n    createdAt\n    errors\n    tokensNeeded\n    entries\n    __typename\n  }\n  total: _allRequestsMeta(page: $page, perPage: $perPage, filter: $filter) {\n    count\n    __typename\n  }\n}"}
     *
     * Filtering requests:
     * {"operationName":"allRequests","variables":{"filter":{"templateIdEq":1,"stateEq":"signed"},"page":0,"perPage":20,"sortField":"id","sortOrder":"DESC"},"query":"query allRequests($page: Int, $perPage: Int, $sortField: String, $sortOrder: String, $filter: RequestFilter) {\n  items: allRequests(\n    page: $page\n    perPage: $perPage\n    sortField: $sortField\n    sortOrder: $sortOrder\n    filter: $filter\n  ) {\n    id\n    templateId\n    templateName\n    templateKind\n    state\n    name\n    createdAt\n    errors\n    tokensNeeded\n    entries\n    __typename\n  }\n  total: _allRequestsMeta(page: $page, perPage: $perPage, filter: $filter) {\n    count\n    __typename\n  }\n}"}
     */
  }
}

