use super::*;
use serde::{Deserialize, Serialize};
use models::wizard::*;


#[derive(GraphQLObject)]
#[graphql(description = "An html preview of an entry's contents")]
pub struct Preview{
  #[graphql(description = "number identifying the entry")]
  pub id: i32,
  #[graphql(description = "entry in html format")]
  pub html: String
}

#[derive(GraphQLInputObject, Serialize, Deserialize)]
#[graphql(description = "WizardInput Object")]
#[serde(rename_all = "camelCase")]
pub struct WizardInput {
  #[graphql(description = "csv file with which the entries will be created")]
  csv: String,
  #[graphql(description = "name of the issuance that will be created")]
  name: String,
  #[graphql(description = "id of the template, if any")]
  template_id: Option<i32>,
  #[graphql(description = "kind of template to be created if there is no template")]
  new_kind: Option<models::TemplateKind>,
  #[graphql(description = "name of template to be created if there is no template")]
  new_name: Option<String>,
  #[graphql(description = "text to be used as the template logo if there is no template")]
  new_logo_text: Option<String>,
  #[graphql(description = "an image to be used as the template logo, insted of text")]
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

    let db_request = Wizard {
      person: person,
      csv: self.csv.into_bytes().clone(),
      name: self.name,
      template
    }.process().await?;

    Ok(Issuance::db_to_graphql(db_request, false).await?)
  }
}
