use super::*;
use serde::{Deserialize, Serialize};
use models::wizard::*;


#[derive(GraphQLObject)]
#[graphql(description = "An html preview of an entry's contents")]
pub struct Preview{ pub id: i32, pub html: String }

#[derive(GraphQLInputObject, Serialize, Deserialize)]
#[graphql(description = "Request Wizard Input")]
#[serde(rename_all = "camelCase")]
pub struct WizardInput {
  csv: String,
  name: String,
  template_id: Option<i32>,
  new_kind: Option<models::TemplateKind>,
  new_name: Option<String>,
  new_logo_text: Option<String>,
  new_logo_image: Option<String>,
}

impl WizardInput {
  pub async fn create_wizard(self, context: &Context) -> FieldResult<Request> {
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

    Ok(Request::db_to_graphql(db_request, false).await?)
  }
}
