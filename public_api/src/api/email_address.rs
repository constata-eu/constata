use super::*;
use db::*;

#[derive(Clone, GraphQLInputObject, Serialize, Deserialize)]
#[graphql(description = "Input data object to register a new email address or change the visibility of a current one. ")]
#[serde(rename_all = "camelCase")]
pub struct EmailAddressInput {
  #[graphql(description = "email to be registered by the person")]
  pub address: String,
  #[graphql(description = "boolean pointing out whether the email should be registered as private or could be public")]
  pub keep_private: bool,
}

impl EmailAddressInput {
  pub async fn process(self, context: &Context) -> FieldResult<EmailAddress> {
    let person = context.site.person().find(context.person_id()).await?;
    let address = person.create_or_update_email_address(&self.address, self.keep_private).await?;
    person.state.outgoing_email_message().create(&person, &address, OutgoingEmailMessageKind::EmailVerification).await?;
    Ok(EmailAddress::db_to_graphql(address).await?)
  }
}

#[derive(GraphQLObject)]
#[graphql(description = "This resource is used by Constata's frontend when you attempt to verify your email address following a link we send to your email.")]
pub struct EmailAddressVerification {
  #[graphql(description = "Unique id of the email address that was verified")]
  pub id: i32,
}

#[derive(GraphQLObject)]
#[graphql(description = "This object show an email address information")]
pub struct EmailAddress {
  #[graphql(description = "number identifying this email address")]
  id: i32,
  #[graphql(description = "Id of the Person ownining this address")]
  person_id: i32,
  #[graphql(description = "The actual email address.")]
  address: String,
  #[graphql(description = "Date when the email was verified, if verified.")]
  verified_at: Option<UtcDateTime>,
  #[graphql(description = "Whether constata should also make this address part of the person's public signature information")]
  keep_private: bool,
}

#[derive(Clone, GraphQLInputObject, Debug)]
pub struct EmailAddressFilter {
  ids: Option<Vec<i32>>,
  id_eq: Option<i32>,
  address_eq: Option<String>,
  person_id_eq: Option<i32>,
}

#[rocket::async_trait]
impl Showable<db::EmailAddress, EmailAddressFilter> for EmailAddress {
  fn sort_field_to_order_by(field: &str) -> Option<EmailAddressOrderBy> {
    match field {
      "id" => Some(EmailAddressOrderBy::Id),
      "verifiedAt" => Some(EmailAddressOrderBy::VerifiedAt),
      _ => None,
    }
  }

  fn filter_to_select(org_id: i32, filter: Option<EmailAddressFilter>) -> SelectEmailAddress {
    if let Some(f) = filter {
      SelectEmailAddress {
        org_id_eq: Some(org_id),
        id_in: f.ids,
        id_eq: f.id_eq,
        person_id_eq: f.person_id_eq,
        address_eq: f.address_eq,
        ..Default::default()
      }
    } else {
      SelectEmailAddress {
        org_id_eq: Some(org_id),
        ..Default::default()
      }
    }
  }

  fn select_by_id(org_id: i32, id: i32) -> SelectEmailAddress {
    SelectEmailAddress { id_eq: Some(id), org_id_eq: Some(org_id), ..Default::default() }
  }

  async fn db_to_graphql(d: db::EmailAddress) -> ConstataResult<Self> {
    let a = d.attrs;
    Ok(EmailAddress {
      id: a.id,
      person_id: a.person_id,
      verified_at: a.verified_at,
      address: a.address,
      keep_private: a.keep_private,
    })
  }
}
