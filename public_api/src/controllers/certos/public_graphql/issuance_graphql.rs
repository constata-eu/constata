use super::*;

#[derive(GraphQLInputObject, Serialize, Deserialize)]
#[graphql(description = "A CreateIssuanceInput configures a new Issuance, with at least one recipient, where more new entries may be added later. Once all desired entries have been added, the issuance may be signed and will be certified and optionally distributed by Constata. If you want to create an Issuance all at once from a single CSV file we suggest you use the Wizard endpoint.")]
#[serde(rename_all = "camelCase")]
pub struct CreateIssuanceInput {
  #[graphql(description = "An array of JSON objects corresponding to each recipient for whom you want to create a diploma, certificate of attendance or badge")]
  entries: Vec<serde_json::Json>,
  #[graphql(description = "The name of the Issuance to be created.")]
  name: String,
  #[graphql(description = "The ID of an existing template to use, if any. See the Templates resource.")]
  template_id: Option<i32>,
  #[graphql(description = "The kind of template to be created if no template_id is given.")]
  new_kind: Option<models::TemplateKind>,
  #[graphql(description = "The name of the new template to be created, if no template_id is given.")]
  new_name: Option<String>,
  #[graphql(description = "The text to be used as the logo for the new template, if no template_id is given.")]
  new_logo_text: Option<String>,
  #[graphql(description = "The base64 encoded image to be used as the logo for the new template, if no template_id is given. If you leave it empty your new_logo_text will be displayed.")]
  new_logo_image: Option<String>,
}

impl CreateIssuanceInput {
  pub async fn create_issuance(self, context: &Context) -> FieldResult<Issuance> {
    let template = match self.template_id {
      Some(id) => WizardTemplate::Existing{ template_id: id},
      None => {
        WizardTemplate::New {
          kind: self.new_kind.ok_or_else(|| Error::validation("newKind", "cannot_be_empty"))?,
          name: self.new_name.ok_or_else(|| Error::validation("newName", "cannot_be_empty"))?,
          logo: match self.new_logo_image {
            Some(i) => ImageOrText::Image(base64::decode(i)?),
            _ => ImageOrText::Text(self.new_logo_text.ok_or_else(|| Error::validation("newLogoText", "cannot_be_empty"))?),
          }
        }
      }
    };

    let person = context.site.person().find(context.person_id()).await?;

    let db_request = wizard::Wizard {
      person: person,
      csv: self.csv.into_bytes().clone(),
      name: self.name,
      template
    }.process().await?;

    Ok(Issuance::db_to_graphql(db_request, false).await?)
  }
}

#[derive(GraphQLObject)]
#[graphql(description = "Represents a batch generation and certification of diplomas, proofs of attendance, and badges from a template. Can be done through our Wizard from a CSV file, or incrementally using this API.")]
pub struct Issuance {
    #[graphql(description = "Unique identifier for the issuance.")]
    id: i32,
    #[graphql(description = "Identifier of the template linked to this issuance.")]
    template_id: i32,
    #[graphql(description = "Name of the template linked to this issuance.")]
    template_name: String,
    #[graphql(description = "The kind of template, which can be 'Diploma', 'Attendance', or 'Invitation'.")]
    template_kind: TemplateKind,
    #[graphql(description = "The state of the issuance, which can be 'received': The recipients data has been received, and we're in the process of generating each recipients document; 'created': Individual entries have been generated from the selected template, using each recipient's data. At this point you can still add more recipients which will rewind the state to 'received'; 'signed': You have signed the entries, no further entries can be added. Documents will be certified and notified within 2 hours; 'completed': All entries have been certified and notified; 'failed': An error ocurred in the creation process, and the whole issuance has been aborted. Look at the errors field for more details.")]
    state: String,
    #[graphql(description = "The name of the issuance.")]
    name: String,
    #[graphql(description = "The date on which this issuance was created.")]
    created_at: UtcDateTime,
    #[graphql(description = "Errors that happened in the process of the issuance, if any. When an error occurs, the whole issuance is halted and no documents are certified.")]
    errors: Option<String>,
    #[graphql(description = "Amount of tokens that the user must buy to certify this issuance.")]
    tokens_needed: Option<i32>,
    #[graphql(description = "Entries that belong to this issuance.")]
    entries: Vec<Entry>,
}

#[derive(GraphQLObject)]
#[graphql(description = "An issuance exported as a CSV file. All rows preserve the order of the original CSV file, or the order in which the entries were added through the API. New columns are added with details about each entry.")]
pub struct IssuanceExport {
  #[graphql(description = "Unique identifier of the issuance.")]
  pub id: i32,
  #[graphql(description = "The CSV plain text of the issuance.")]
  pub csv: String,
}

#[derive(Clone, GraphQLInputObject, Debug)]
pub struct IssuanceFilter {
  ids: Option<Vec<i32>>,
  id_eq: Option<i32>,
  template_id_eq: Option<i32>,
  state_eq: Option<String>,
  name_like: Option<String>,
}

#[rocket::async_trait]
impl Showable<request::Request, IssuanceFilter> for Issuance {
  fn sort_field_to_order_by(field: &str) -> Option<RequestOrderBy> {
    match field {
      "id" => Some(RequestOrderBy::Id),
      "templateId" => Some(RequestOrderBy::TemplateId),
      "state" => Some(RequestOrderBy::State),
      "name" => Some(RequestOrderBy::Name),
      "createdAt" => Some(RequestOrderBy::CreatedAt),
      _ => None,
    }
  }

  fn filter_to_select(org_id: i32, f: IssuanceFilter) -> SelectRequest {
    SelectRequest {
      id_in: f.ids,
      org_id_eq: Some(org_id),
      id_eq: f.id_eq,
      template_id_eq: f.template_id_eq,
      state_eq: f.state_eq,
      state_ne: Some("failed".to_string()),
      name_ilike: into_like_search(f.name_like),
      deletion_id_is_set: Some(false),
      ..Default::default()
    }
  }

  fn select_by_id(org_id: i32, id: i32) -> SelectRequest {
    SelectRequest { id_eq: Some(id), org_id_eq: Some(org_id), ..Default::default() }
  }

  async fn db_to_graphql(d: request::Request, _with_payload: bool) -> MyResult<Self> {
    let template = d.template().await?;
    let db_entries = d.entry_vec().await?;
    let tokens_needed = if d.is_created() { Some(d.in_created()?.tokens_needed().await?) }
    else { None };
    let mut entries = vec![];
    for entry in db_entries {
      entries.push(Entry::db_to_graphql(entry, false).await?);
    }
    Ok(Issuance {
      id: d.attrs.id,
      template_id: d.attrs.template_id,
      template_name: template.attrs.name,
      template_kind: template.attrs.kind,
      state: d.attrs.state,
      name: d.attrs.name,
      errors: d.attrs.errors,
      created_at: d.attrs.created_at,
      tokens_needed,
      entries,
    })
  }
}


/*
El workflow del usuario via graphql es:
- Envía un issuance, con entries inline opcionalmente, espera a que esté created.
- Le puede agregar mas entries, eso pasa el issuance de "created" otra vez a "received".
- Si está en created o received puede seguir agregando entries.
- Si fue creado via CSV igual se le puede agregar entries.
- Puede tirar errores de validación del entry que estoy intentando agregar.
- Como el entry se crea asincrónicamente, y pasa el issuance de "created" a "received", es posible que el Issuance quede en failed culpa de un entry y tenga que cargar todo otra vez.
  - Esto va a estar muy mitigado por la validación.
  - En cualquier caso, un Issuance fallido no se puede firmar.

 */
constata_lib::describe_one! {
  fulltest!{ can_create_an_issuance (_site, c, client, mut chain)
    client.signer.verify_email("test@example.com").await;

    /*
    use gql::{
      *,
      create_issuance as create,
      update_issuance as update,
      issuance as show,
      all_issuances as all,
    };


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
    */
  }
}

