use super::*;

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
