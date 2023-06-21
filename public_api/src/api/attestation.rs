use super::*;
use db::{*, attestation::for_api::from_model};
pub use db::attestation::for_api::Attestation;

#[derive(GraphQLInputObject, Serialize, Deserialize)]
#[graphql(
  description = "An AttestationInput has all parameters required to create an Attestation on several of documents."
  scalar=GqlScalar
)]
#[serde(rename_all = "camelCase")]
pub struct AttestationInput {
  #[graphql(description = "An array of SignedPayloads containing all the documents to attest. See the tutorial for more info on signing payloads.")]
  pub documents: Vec<signed_payload::SignedPayload>,
  #[graphql(description = "An attestation allows appending documents up until a certain date. If you don't chose a date, no appending will be allowed.")]
  pub open_until: Option<UtcDateTime>,
  #[graphql(description = "Markers is a text that can be used for searching this attestation later. Markers cannot be updated after creation.")]
  pub markers: Option<String>,
  #[graphql(description = "A list of email addresses to notify when the documents are attested. Constata will email them an administrative access link to view, download or share the document certificate. You can pass an empty list if you want to omit Constata's emails, and manage distribution of the attestation in any other way.")]
  pub email_admin_access_url_to: Vec<String>,
}

impl AttestationInput {
  pub async fn create_attestation(self, context: &Context) -> FieldResult<Attestation> {
    let person = context.person();

    let att = context.site.attestation()
      .create(&person, &self.documents, self.open_until, self.markers, Some(context.lang), self.email_admin_access_url_to)
      .await?;

    Ok(Attestation::db_to_graphql(att).await?)
  }
}

#[derive(Debug, Clone, GraphQLObject, Serialize, Deserialize)]
#[graphql(description = "You can get an attestation as a verifiable HTML, embedding all documents and verifiable in any default browser.")]
#[serde(rename_all = "camelCase")]
pub struct AttestationHtmlExport {
  pub id: i32,
  pub attestation: Attestation,
  #[graphql(description = "The verifiable HTML proof.")]
  pub verifiable_html: String,
}

#[derive(Debug, Clone, Default, GraphQLInputObject, Serialize, Deserialize, clap::Args)]
#[serde(rename_all = "camelCase")]
pub struct AttestationFilter {
  #[arg(long, help="Fetch a specific list of attestations by their ids", action=clap::ArgAction::Append)]
  ids: Option<Vec<i32>>,
  #[arg(long, help="Fetch a specific attestation by id")]
  id_eq: Option<i32>,
  person_id_eq: Option<i32>,
  #[arg(long, help="Filter attestations that have this text in their markers")]
  markers_like: Option<String>,
}

#[rocket::async_trait]
impl Showable<db::Attestation, AttestationFilter> for Attestation {
  fn sort_field_to_order_by(field: &str) -> Option<AttestationOrderBy> {
    match field {
      "id" => Some(AttestationOrderBy::Id),
      "personId" => Some(AttestationOrderBy::PersonId),
      "createdAt" => Some(AttestationOrderBy::CreatedAt),
      _ => None,
    }
  }

  fn filter_to_select(org_id: i32, filter: Option<AttestationFilter>) -> SelectAttestation {
    if let Some(f) = filter {
      SelectAttestation {
        id_in: f.ids,
        org_id_eq: Some(org_id),
        id_eq: f.id_eq,
        person_id_eq: f.person_id_eq,
        markers_ilike: into_like_search(f.markers_like),
        deletion_id_is_set: Some(false),
        ..Default::default()
      }
    } else {
      SelectAttestation {
        org_id_eq: Some(org_id),
        deletion_id_is_set: Some(false),
        ..Default::default()
      }
    }
  }

  fn select_by_id(org_id: i32, id: i32) -> SelectAttestation {
    SelectAttestation { id_eq: Some(id), org_id_eq: Some(org_id), ..Default::default() }
  }

  async fn db_to_graphql(d: db::Attestation) -> ConstataResult<Self> {
    Ok(from_model(d).await?)
  }
}

#[derive(Debug, Clone, Default, GraphQLInputObject, Serialize, Deserialize, clap::Args)]
#[graphql(description = "Used to publish/unpubish an attestation, so that anyone can see it hosted in Constata's website")]
#[serde(rename_all = "camelCase")]
pub struct AttestationSetPublishedInput {
  #[arg(help="The attestation to publish/unpublish")]
  pub attestation_id: i32,
  #[arg(short, long, help="Pass this flag to publish. Omit this flag to unpublish.")]
  pub publish: bool,
}

impl AttestationSetPublishedInput {
  pub async fn process(self, context: &Context) -> FieldResult<Attestation> {
    let attestation = context.person().org().await?.attestation_scope().id_eq(&self.attestation_id).one().await?;
    if let Some(link) = attestation.story().await?.get_or_create_download_proof_link(30).await? {
      if self.publish {
        link.publish().await?;
      } else {
        link.unpublish().await?;
      }
      Ok(Attestation::db_to_graphql(attestation).await?)
    } else {
      Err(field_error("not_ready_to_publish", "cannot_publish_until_done"))
    }
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
      attestation_set_published as publish,
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
        public_certificate_url: eq(None),
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

    let published: publish::ResponseData = client.gql(
      &AttestationSetPublished::build_query(publish::Variables{
        input: publish::AttestationSetPublishedInput{ attestation_id: 1, publish: true }
      })
    ).await;

    assert_that!(
      &published.attestation_set_published.public_certificate_url.unwrap(),
      rematch("http://localhost:8000/certificate/")
    );

    let unpublished: publish::ResponseData = client.gql(
      &AttestationSetPublished::build_query(publish::Variables{
        input: publish::AttestationSetPublishedInput{ attestation_id: 1, publish: false }
      })
    ).await;

    assert!(unpublished.attestation_set_published.public_certificate_url.is_none());

    let bob_client = crate::test_support::PublicApiClient::new(c.bob().await).await;
    let empty_search = all::Variables{
      page: None,
      sort_field: None,
      per_page: None,
      sort_order: None,
      filter: None,
    };
    let nothing: all::ResponseData = bob_client.gql(&AllAttestations::build_query(empty_search)).await;
    assert!(&nothing.all_attestations.is_empty());
  }
}
