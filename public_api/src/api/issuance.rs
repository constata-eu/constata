use super::*;
use db::*;
use std::collections::HashMap;
use serde_with::base64::Base64;
use juniper::{InputValue, ScalarValue, Value, ScalarToken, ParseScalarResult, ParseScalarValue};

#[rocket::async_trait]
pub trait CreateIssuanceInput: Send + Sized {
  async fn create(self, person: Person, template: WizardTemplate) -> ConstataResult<db::Issuance>;
  fn attrs(&self) -> (&Option<i32>, &Option<TemplateKind>, &Option<String>, &Option<String>, &Option<Vec<u8>>);

  fn required<T: Clone>(val: &Option<T>, name: &str) -> ConstataResult<T> {
    val.clone().ok_or_else(|| Error::validation(name, "cannot_be_empty"))
  }

  async fn process(self, context: &Context) -> FieldResult<Issuance> {
    let (template_id, new_kind, new_name, new_logo_text, new_logo_image) = self.attrs();

    let template = match template_id {
      Some(id) => WizardTemplate::Existing{ template_id: *id},
      None => {
        WizardTemplate::New {
          kind: Self::required(new_kind, "newKind")?,
          name: Self::required(new_name, "newName")?,
          logo: if let Some(i) = new_logo_image {
            ImageOrText::Image(i.clone())
          } else {
            ImageOrText::Text(Self::required(new_logo_text, "newLogoText")?)
          }
        }
      }
    };

    let person = context.site.person().find(context.person_id()).await?;
    Ok(Issuance::db_to_graphql(self.create(person, template).await?).await?)
  }
}

#[serde_as]
#[derive(GraphQLInputObject, Serialize)]
#[graphql(
  description = "A CreateIssuanceFromJsonInput configures a new Issuance, with optional initial entries, where more new entries may be added later. Once all desired entries have been added, the issuance may be signed and will be certified and optionally distributed by Constata. If you want to create an Issuance all at once from a single CSV file we suggest you use CreateIssuanceFromCsvInput."
  scalar=GqlScalar
)]
#[serde(rename_all = "camelCase")]
#[derive(clap::Args)]
pub struct CreateIssuanceFromJsonInput {
  #[arg(short, long="entry", value_name="ENTRY", value_parser=clap_entry_params, action=clap::ArgAction::Append,
    help="A JSON object corresponding to each recipient for whom you want to create a diploma, certificate of attendance or badge. \
      This is a nice shortcut for issuances with just a few, small entries. \
      See the append-entries-to-issuance command for incrementally building an issuance from larger entries.")]
  #[graphql(description = "An array of JSON objects corresponding to each recipient for whom you want to create a diploma, certificate of attendance or badge")]
  pub entries: Vec<EntryParams>,

  #[arg(help="The name of the Issuance to be created")]
  #[graphql(description = "The name of the Issuance to be created.")]
  pub name: String,

  #[arg(short, long, help="The kind of template to be created if no template_id is given.")]
  #[graphql(description = "The ID of an existing template to use, if any. See the allTemplates query.")]
  pub template_id: Option<i32>,

  #[arg(long, help="The kind of template to be created if no template_id is given.")]
  #[graphql(description = "The kind of template to be created if no template_id is given.")]
  pub new_kind: Option<TemplateKind>,

  #[arg(long, help="The name of the new template to be created, if no template_id is given.")]
  #[graphql(description = "The name of the new template to be created, if no template_id is given.")]
  pub new_name: Option<String>,

  #[arg(long, help="The text to be used as the logo for the new template, if no template_id is given.")]
  #[graphql(description = "The text to be used as the logo for the new template, if no template_id is given.")]
  pub new_logo_text: Option<String>,

  #[arg(long, help="A base64 encoded image to use as your logo. \
    Use --new-logo-image-file to use a local file instead. \
    If you leave it empty your new_logo_text will be displayed."
  )]
  #[graphql(description = "The base64 encoded image to be used as the logo for the new template, if no template_id is given. If you leave it empty your new_logo_text will be displayed.")]
  #[serde_as(as = "Option<Base64>")]
  pub new_logo_image: Option<Bytes>,
}

#[rocket::async_trait]
impl CreateIssuanceInput for CreateIssuanceFromJsonInput {
  async fn create(self, person: Person, template: WizardTemplate) -> ConstataResult<db::Issuance> {
    JsonIssuanceBuilder {
      person: person,
      entries: self.entries.into_iter().map(|e| e.0).collect(),
      name: self.name.clone(),
      template
    }.process().await
  }

  fn attrs(&self) -> (&Option<i32>, &Option<TemplateKind>, &Option<String>, &Option<String>, &Option<Vec<u8>>) {
    (&self.template_id, &self.new_kind, &self.new_name, &self.new_logo_text, &self.new_logo_image)
  }
}

#[derive(Clone, Debug, juniper::GraphQLScalar, serde::Serialize, serde::Deserialize)]
pub struct EntryParams(#[serde(serialize_with="as_json")] pub HashMap<String, String>);

fn as_json<T, S>(v: &T, serializer: S) -> Result<S::Ok, S::Error>
  where T: Serialize, S: serde::ser::Serializer,
{
  let as_str = serde_json::to_string(&v).unwrap();
  serializer.serialize_str(&as_str)
}

fn clap_entry_params(s: &str) -> Result<EntryParams, String> {
  serde_json::from_str(&s)
    .map_err(|_| format!("EntryParams should be a json object with only strings as its values, it was: {}", &s))
}

impl EntryParams {
  fn from_input<S>(v: &InputValue<S>) -> Result<Self, String> where S: ScalarValue {
    let string = v.as_string_value()
      .ok_or_else(|| "value was not serializable".to_string())?;
    let vec: HashMap<String,String> = serde_json::from_str(&string)
      .map_err(|_| format!("EntryParams should be a json object with only strings as its values, it was: {}", &string))?;

    Ok(EntryParams(vec))
  }

  fn to_output<S: ScalarValue>(&self) -> Value<S> {
    Value::scalar(serde_json::to_string(&self.0).unwrap())
  }

  fn parse_token<S>(value: ScalarToken<'_>) -> ParseScalarResult<S> where S: ScalarValue {
    <String as ParseScalarValue<S>>::from_str(value)
  }
}

#[serde_as]
#[derive(GraphQLInputObject, Serialize)]
#[graphql(
  description = "A CreateIssuanceFromCsvInput configures a new Issuance, with at least one recipient, where more new entries may be added later. Once all desired entries have been added, the issuance may be signed and will be certified and optionally distributed by Constata. If you want to create an Issuance all at once from a single CSV file we suggest you use the Wizard endpoint.",
  scalar=GqlScalar
)]
#[serde(rename_all = "camelCase")]
#[derive(clap::Args)]
pub struct CreateIssuanceFromCsvInput {
  #[graphql(description = "The CSV string to be used for creating the issuance")]
  #[arg(long, default_value="", help="The CSV string to be used for creating the issuance. Use --csv-file to use a local file instead")]
  pub csv: String,

  #[arg(help="The name of the Issuance to be created")]
  #[graphql(description = "The name of the Issuance to be created.")]
  pub name: String,

  #[arg(short, long, help="The kind of template to be created if no template_id is given.")]
  #[graphql(description = "The ID of an existing template to use, if any. See the Templates resource.")]
  pub template_id: Option<i32>,

  #[arg(long, help="The kind of template to be created if no template_id is given.")]
  #[graphql(description = "The kind of template to be created if no template_id is given.")]
  pub new_kind: Option<TemplateKind>,

  #[arg(long, help="The name of the new template to be created, if no template_id is given.")]
  #[graphql(description = "The name of the new template to be created, if no template_id is given.")]
  pub new_name: Option<String>,

  #[arg(long, help="The text to be used as the logo for the new template, if no template_id is given.")]
  #[graphql(description = "The text to be used as the logo for the new template, if no template_id is given.")]
  pub new_logo_text: Option<String>,

  #[arg(long, help="A base64 encoded image to use as your logo. \
    Use --new-logo-image-file to use a local file instead. \
    If you leave it empty your new_logo_text will be displayed."
  )]
  #[graphql(description = "The base64 encoded image to be used as the logo for the new template, if no template_id is given. If you leave it empty your new_logo_text will be displayed.")]
  #[serde_as(as = "Option<Base64>")]
  pub new_logo_image: Option<Bytes>,
}

#[rocket::async_trait]
impl CreateIssuanceInput for CreateIssuanceFromCsvInput {
  async fn create(self, person: Person, template: WizardTemplate) -> ConstataResult<db::Issuance> {
    Wizard {
      person: person,
      csv: self.csv.into_bytes().clone(),
      name: self.name.clone(),
      template
    }.process().await
  }

  fn attrs(&self) -> (&Option<i32>, &Option<TemplateKind>, &Option<String>, &Option<String>, &Option<Vec<u8>>) {
    (&self.template_id, &self.new_kind, &self.new_name, &self.new_logo_text, &self.new_logo_image)
  }
}

#[derive(Clone, GraphQLInputObject, Serialize)]
#[graphql(description = "This is the best way to compose an Issuance incrementally. You can add new entries at any time before signing the Issuance. Entries will be validated as they are recevied, and then will be 'created' by our workers. In the unlikely case an entry passes validation and is received, but then an error when our worker tries to create it, the issuance will be marked as failed, as well as all other entries. ")]
#[serde(rename_all = "camelCase")]
#[derive(clap::Args)]
pub struct AppendEntriesToIssuanceInput {
  #[arg(help="The ID of the issuance to which the entries are to be appended")]
  #[graphql(description = "The ID of the Issuance to which the entries are to be appended.")]
  pub issuance_id: i32,

  #[arg(short, long="entry", value_name="ENTRY", value_parser=clap_entry_params, action=clap::ArgAction::Append,
    help="A flat JSON objects with strings as its keys and values, to be used as parameters for your entry. You can repeat this argument. \
      ie: '{\"name\":\"Bob\",\"motive\":\"Accredited Expert\"}'")]
  #[graphql(description = "An array of JSON objects corresponding to each recipient for whom you want to create a diploma, \
      certificate of attendance or badge. \
      ie: '[{\"name\":\"Alice\",\"motive\":\"Cream of the crop\"},{\"name\":\"Bob\",\"motive\":\"Accredited Expert\"}]'")]
  pub entries: Vec<EntryParams>,
}

impl AppendEntriesToIssuanceInput {
  pub async fn process(self, context: &Context) -> FieldResult<Issuance> {
    let issuance = context.person().org().await?.issuance_scope().id_eq(&self.issuance_id).one().await?;
    if let Some(received) = issuance.flow().for_adding_entries().await? {
      received.append_entries(
        &self.entries.into_iter().map(|e| e.0).collect::<Vec<HashMap<String,String>>>(),
      ).await?;
      Ok(Issuance::db_to_graphql(received.into_inner()).await?)
    } else {
      Err(field_error("already_signing", "cannot_append_entries_at_this_point"))
    }
  }
}

#[derive(Debug, GraphQLObject, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[graphql(description = "Represents a batch generation and certification of diplomas, proofs of attendance, and badges from a template. Can be started from a CSV file using CreateIssuanceFromCsv, or from json directly using CreateIssuanceFromJson.")]
pub struct Issuance {
  #[graphql(description = "Unique identifier for the issuance.")]
  pub id: i32,
  #[graphql(description = "Identifier of the template linked to this issuance.")]
  pub template_id: i32,
  #[graphql(description = "Name of the template linked to this issuance.")]
  pub template_name: String,
  #[graphql(description = "The kind of template, which can be 'DIPLOMA', 'ATTENDANCE', or 'BADGE'.")]
  pub template_kind: TemplateKind,
  #[graphql(description = "The state of the issuance, which can be 'received': The recipients data has been received, and we're in the process of generating each recipients document; 'created': Individual entries have been generated from the selected template, using each recipient's data. At this point you can still add more recipients which will rewind the state to 'received'; 'signed': You have signed the entries, no further entries can be added. Documents will be certified and notified within 2 hours; 'completed': All entries have been certified and notified; 'failed': An error ocurred in the creation process, and the whole issuance has been aborted. Look at the errors field for more details.")]
  pub state: String,
  #[graphql(description = "The name of the issuance.")]
  pub name: String,
  #[graphql(description = "The date on which this issuance was created.")]
  pub created_at: UtcDateTime,
  #[graphql(description = "Errors that happened in the process of the issuance, if any. When an error occurs, the whole issuance is halted and no documents are certified.")]
  pub errors: Option<String>,
  #[graphql(description = "Amount of tokens that the user must buy to certify this issuance.")]
  pub tokens_needed: Option<i32>,
  #[graphql(description = "Entry count for this issuance. All entries can be fetch separately with an Entries query, filtering by issuance id.")]
  pub entries_count: i32,
  #[graphql(description = "Stats: How many recipients viewed the admin link that was sent to them.")]
  pub admin_visited_count: i32,
  #[graphql(description = "Stats: How many visits did the published entries in this Issuance get, collectively.")]
  pub public_visit_count: i32, 
}

#[derive(Debug, Clone, GraphQLObject, serde::Serialize, serde::Deserialize)]
#[graphql(description = "An issuance exported as a CSV file. All rows preserve the order of the original CSV file, or the order in which the entries were added through the API. New columns are added with details about each entry.")]
pub struct IssuanceExport {
  #[graphql(description = "Unique identifier of the issuance.")]
  pub id: i32,
  #[graphql(description = "The CSV plain text of the issuance.")]
  pub csv: String,
}

#[derive(Debug, Clone, Default, GraphQLInputObject, serde::Serialize, serde::Deserialize, clap::Args)]
#[serde(rename_all = "camelCase")]
pub struct IssuanceFilter {
  #[arg(long, help="Fetch a specific list of issuances by their ids", action=clap::ArgAction::Append)]
  pub ids: Option<Vec<i32>>,
  #[arg(long, help="Fetch a specific issuance by id")]
  pub id_eq: Option<i32>,
  #[arg(long, help="Filter by template id")]
  pub template_id_eq: Option<i32>,
  #[arg(long, help="Filter by state: 'received', 'created', 'signed', 'completed', 'failed'")]
  pub state_eq: Option<String>,
  #[arg(long, help="Filter where name contains this text")]
  pub name_like: Option<String>,
}

#[rocket::async_trait]
impl Showable<db::Issuance, IssuanceFilter> for Issuance {
  fn sort_field_to_order_by(field: &str) -> Option<IssuanceOrderBy> {
    match field {
      "id" => Some(IssuanceOrderBy::Id),
      "templateId" => Some(IssuanceOrderBy::TemplateId),
      "state" => Some(IssuanceOrderBy::State),
      "name" => Some(IssuanceOrderBy::Name),
      "createdAt" => Some(IssuanceOrderBy::CreatedAt),
      _ => None,
    }
  }

  fn filter_to_select(org_id: i32, filter: Option<IssuanceFilter>) -> SelectIssuance {
    if let Some(f) = filter {
      SelectIssuance {
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
    } else {
      SelectIssuance {
        org_id_eq: Some(org_id),
        deletion_id_is_set: Some(false),
        ..Default::default()
      }
    }
  }

  fn select_by_id(org_id: i32, id: i32) -> SelectIssuance {
    SelectIssuance { id_eq: Some(id), org_id_eq: Some(org_id), ..Default::default() }
  }

  async fn db_to_graphql(d: db::Issuance) -> ConstataResult<Self> {
    let template = d.template().await?;
    let db_entries = d.entry_vec().await?;
    let tokens_needed = if d.is_created() { Some(d.in_created()?.tokens_needed().await?) } else { None };

    let mut admin_visited_count = 0;
    let mut public_visit_count = 0;
    for entry in &db_entries {
      let Some(document) = entry.document().await? else { continue; };
      let Some(l) = document.download_proof_link_scope().optional().await? else { continue; };
      if l.attrs.admin_visited { admin_visited_count += 1 };
      public_visit_count += l.attrs.public_visit_count;
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
      entries_count: db_entries.len() as i32,
      admin_visited_count,
      public_visit_count,
    })
  }
}

/*
constata_lib::describe_one! {
  fulltest!{ can_create_an_issuance (site, c, client)
    client.signer.verify_email("test@example.com").await;

    use gql::{
      *,
      create_issuance_from_json as create,
      issuance as show,
      all_entries,
      append_entries_to_issuance as append,
    };

    let entries = vec![
      serde_json::json!({
        "name": "Kenny Mcormic",
        "email": "kenny@cc.com",
        "recipient_identification": "AB-12345",
        "custom_text": "Artísta plastilínico",
        "motive": "Arte con plastilina",
        "date": "3 de marzo de 1999",
        "place": "Sout Park",
        "shared_text": "Gracias a todos por venir",
      }).to_string(),
      serde_json::json!({
        "name": "Kyle Broflovsky",
        "email": "kyle@cc.com",
        "recipient_identification": "AB-12345",
        "custom_text": "Artísta plastilínico",
        "motive": "Arte con plastilina",
        "date": "3 de marzo de 1999",
        "place": "Sout Park",
        "shared_text": "Gracias a todos por venir",
      }).to_string(),
    ];

    let vars = create::Variables{
      input: create::CreateIssuanceFromJsonInput {
        entries,
        name: "testing".to_string(),
        template_id: None,
        new_kind: Some(create::TemplateKind::DIPLOMA),
        new_name: Some("nuevo diploma".to_string()),
        new_logo_text: Some("nuevo texto del logo".to_string()),
        new_logo_image: None::<String>,
      }
    };

    let received: create::ResponseData = client.gql(&CreateIssuanceFromJson::build_query(vars)).await;

    assert_that!(&received, structure!{ create::ResponseData {
      create_issuance_from_json: structure! { create::CreateIssuanceFromJsonCreateIssuanceFromJson {
        id: eq(1),
        template_id: eq(1),
        template_name: rematch("nuevo diploma"),
        template_kind: eq(create::TemplateKind::DIPLOMA),
        state: rematch("received"),
        name: rematch("testing"),
        errors: eq(None),
        tokens_needed: eq(None),
        entries_count: eq(2),
        admin_visited_count: eq(0),
        public_visit_count: eq(0),
      }}
    }});

    site.issuance().create_all_received().await?;

    let created: show::ResponseData = client.gql(&Issuance::build_query(show::Variables{ id: 1 })).await;

    assert_that!(&created, structure!{ show::ResponseData {
      issuance: structure! { show::IssuanceIssuance {
        id: eq(1),
        state: rematch("created"),
      }}
    }});

    let new_entries = vec![
      serde_json::json!({
        "name": "Kenny Mcormic",
        "email": "kenny@cc.com",
        "recipient_identification": "AB-12345",
        "custom_text": "Artísta plastilínico",
        "motive": "Arte con plastilina",
        "date": "3 de marzo de 1999",
        "place": "Sout Park",
        "shared_text": "Gracias a todos por venir",
      }).to_string(),
      serde_json::json!({
        "name": "Kyle Broflovsky",
        "email": "kyle@cc.com",
        "recipient_identification": "AB-12345",
        "custom_text": "Artísta plastilínico",
        "motive": "Arte con plastilina",
        "date": "3 de marzo de 1999",
        "place": "Sout Park",
        "shared_text": "Gracias a todos por venir",
      }).to_string(),
    ];

    let vars = append::Variables{
      input: append::AppendEntriesToIssuanceInput {
        issuance_id: 1,
        entries: new_entries,
      }
    };

    let appended: append::ResponseData = client.gql(&AppendEntriesToIssuance::build_query(vars)).await;

    assert_that!(&appended, structure!{ append::ResponseData {
      append_entries_to_issuance: structure! { append::AppendEntriesToIssuanceAppendEntriesToIssuance {
        id: eq(1),
        template_id: eq(1),
        template_name: rematch("nuevo diploma"),
        entries_count: eq(4),
      }}
    }});

    let entries: all_entries::ResponseData = client.gql(&AllEntries::build_query(entries_vars(1))).await;
    assert_that!(&entries.all_entries, all_elements_satisfy(|e: &all_entries::AllEntriesAllEntries| !e.is_published && e.public_certificate_url.is_none()));
  }

  fulltest!{ issuance_entries_have_the_public_certificate_url (site, c, client, mut chain)
    client.signer.verify_email("test@example.com").await;

    use gql::{
      *,
      create_issuance_from_json as create,
      issuance as show,
    };

    let entries = vec![
      serde_json::json!({
        "name": "Kenny Mcormic",
        "recipient_identification": "AB-12345",
        "custom_text": "Artísta plastilínico",
        "motive": "Arte con plastilina",
        "date": "3 de marzo de 1999",
        "place": "Sout Park",
        "shared_text": "Gracias a todos por venir",
      }).to_string(),
    ];

    let vars = create::Variables{
      input: create::CreateIssuanceFromJsonInput {
        entries,
        name: "testing".to_string(),
        template_id: None,
        new_kind: Some(create::TemplateKind::DIPLOMA),
        new_name: Some("nuevo diploma".to_string()),
        new_logo_text: Some("nuevo texto del logo".to_string()),
        new_logo_image: None::<String>,
      }
    };

    let _received: create::ResponseData = client.gql(&CreateIssuanceFromJson::build_query(vars)).await;
    site.issuance().create_all_received().await?;

    let created: show::ResponseData = client.gql(&Issuance::build_query(show::Variables{ id: 1 })).await;

    assert_that!(&created, structure!{ show::ResponseData {
      issuance: structure! { show::IssuanceIssuance {
        id: eq(1),
        state: rematch("created"),
      }}
    }});

    let mut issuance = site.issuance().find(created.issuance.id as i32).await.unwrap();
    let mut signature = None;
    while let Some(next) = issuance.in_created()?.signing_iterator(signature).await? {
      signature = Some(client.signer.sign_issuance_entry(next).await);
    }

    chain.fund_signer_wallet();
    chain.simulate_stamping().await;
    site.issuance().try_complete().await?;
    issuance.reload().await?;
    assert!(issuance.is_completed());
    let entries: all_entries::ResponseData = client.gql(&AllEntries::build_query(entries_vars(1))).await;
    assert_that!(&entries.all_entries, all_elements_satisfy(|e: &all_entries::AllEntriesAllEntries| e.is_published && e.public_certificate_url.is_some()));
  }

    use crate::test_support::gql::all_entries;
    fn entries_vars(issuance_id: i64) -> all_entries::Variables {
        all_entries::Variables{
          filter: Some(all_entries::EntryFilter {
            issuance_id_eq: Some(issuance_id),
            ids: None,
            id_eq: None,
            state_eq: None,
            document_id_eq: None,
            params_like: None
          }),
          page: None,
          per_page: None,
          sort_field: None,
          sort_order: None,
        }
    }
}
*/
