use super::*;
use serde::{Deserialize, Serialize};
use models::wizard::{self, WizardTemplate, ImageOrText};

#[derive(GraphQLObject)]
#[graphql(description = "Represents an HTML preview of the contents of an entry.")]
pub struct Preview{
  #[graphql(description = "The numerical identifier of the entry.")]
  pub id: i32,
  #[graphql(description = "The HTML formatted contents of the entry.")]
  pub html: String
}

#[derive(GraphQLInputObject, Serialize, Deserialize)]
#[graphql(description = "Input object used to create an Issuance from scratch, either with a new template or an existing one.")]
#[serde(rename_all = "camelCase")]
pub struct WizardInput {
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

impl WizardInput {
  pub async fn create_wizard(self, context: &Context) -> FieldResult<Issuance> {
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
