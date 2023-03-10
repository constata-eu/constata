use super::*;
use std::collections::HashMap;
use constata_lib::models::certos::wizard::*;

#[derive(GraphQLInputObject, Serialize, Deserialize)]
#[graphql(description = "A CreateIssuanceFromJsonInput configures a new Issuance, with optional initial entries, where more new entries may be added later. Once all desired entries have been added, the issuance may be signed and will be certified and optionally distributed by Constata. If you want to create an Issuance all at once from a single CSV file we suggest you use CreateIssuanceFromCsvInput.")]
#[serde(rename_all = "camelCase")]
pub struct CreateIssuanceFromJsonInput {
  #[graphql(description = "An array of JSON objects corresponding to each recipient for whom you want to create a diploma, certificate of attendance or badge")]
  entries: Vec<HashMap<String,String>>,
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

impl CreateIssuanceFromJsonInput {
  pub async fn process(self, context: &Context) -> FieldResult<Issuance> {
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

    let db_request = JsonIssuanceBuilder {
      person: person,
      entries: self.entries,
      name: self.name,
      template
    }.process().await?
    .into_inner();

    Ok(Issuance::db_to_graphql(db_request, false).await?)
  }
}

#[derive(GraphQLInputObject, Serialize, Deserialize)]
#[graphql(description = "A CreateIssuanceFromCsvInput configures a new Issuance, with at least one recipient, where more new entries may be added later. Once all desired entries have been added, the issuance may be signed and will be certified and optionally distributed by Constata. If you want to create an Issuance all at once from a single CSV file we suggest you use the Wizard endpoint.")]
#[serde(rename_all = "camelCase")]
pub struct  CreateIssuanceFromCsvInput {
  #[graphql(description = "The CSV file to be used for creating the entries.")]
  csv: String,
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

impl CreateIssuanceFromCsvInput {
  pub async fn process(self, context: &Context) -> FieldResult<Issuance> {
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

    let db_request = Wizard {
      person: person,
      csv: self.csv.into_bytes().clone(),
      name: self.name,
      template
    }.process().await?;

    Ok(Issuance::db_to_graphql(db_request, false).await?)
  }
}

#[derive(GraphQLInputObject, Serialize, Deserialize)]
#[graphql(description = "This is the best way to compose an Issuance incrementally. You can add new entries at any time before signing the Issuance. Entries will be validated as they are recevied, and then will be 'created' by our workers. In the unlikely case an entry passes validation and is received, but then an error when our worker tries to create it, the request will be marked as failed, as well as all other entries. ")]
#[serde(rename_all = "camelCase")]
pub struct AppendEntriesToIssuanceInput {
  #[graphql(description = "The ID of the Issuance to which the entries are to be added.")]
  issuance_id: i32,
  #[graphql(description = "An array of JSON objects corresponding to each recipient for whom you want to create a diploma, certificate of attendance or badge")]
  entries: Vec<HashMap<String,String>>,
}

impl AppendEntriesToIssuanceInput {
  pub async fn process(self, context: &Context) -> FieldResult<Issuance> {
    let issuance = context.person().org().await?.request_scope().id_eq(&context.issuance_id).one().await?;
    if let Some(received) = issuance.flow().for_adding_entries().await? {
      received.append_entries(&self.entries).await?;
      Ok(Issuance::db_to_graphql(received.into_inner(), false).await?)
    } else {
      Err(field_error("already_signing", "cannot_append_entries_at_this_point"))
    }
  }
}

#[derive(GraphQLObject)]
#[graphql(description = "Represents a batch generation and certification of diplomas, proofs of attendance, and badges from a template. Can be started from a CSV file using CreateIssuanceFromCsv, or from json directly using CreateIssuanceFromJson.")]
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
    #[graphql(description = "Stats: How many recipients viewed the admin link that was sent to them.")]
    admin_visited_count: i32,
    #[graphql(description = "Stats: How many visits did the published entries in this Issuance get, collectively.")]
    public_visit_count: i32, 
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
    let tokens_needed = if d.is_created() { Some(d.in_created()?.tokens_needed().await?) } else { None };

    let mut admin_visited_count = 0;
    let mut public_visit_count = 0;
    for entry in db_entries.iter() {
      let Some(document) = entry.document().await? else { continue; };
      let Some(l) = document.download_proof_link_scope().optional().await? else { continue; };
      if l.attrs.admin_visited { admin_visited_count += 1 };
      public_visit_count += l.attrs.public_visit_count;
    }

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
      admin_visited_count,
      public_visit_count,
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

    use gql::{
      *,
      create_issuance_from_json as create,
      append_entries_to_issuance as append,
      issuance as show,
      all_issuances as all,
    };

  }
}

