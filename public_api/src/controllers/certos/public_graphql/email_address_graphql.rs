use super::*;
use serde::{Deserialize, Serialize};
use models::{email_address, outgoing_email_message_kind::*};

#[derive(Clone, GraphQLInputObject, Serialize, Deserialize)]
#[graphql(description = "Email Address Input Object: {
  email: email to be registered by the person
  keep_private: boolean pointing out whether the email should be registered as private or could be public,
}")]
#[serde(rename_all = "camelCase")]
pub struct EmailAddressInput {
  pub address: String,
  pub keep_private: bool,
}

impl EmailAddressInput {
  pub async fn process(self, context: &Context) -> FieldResult<EmailAddress> {
    let person = context.site.person().find(context.person_id()).await?;
    let address = person.create_or_update_email_address(&self.address, self.keep_private).await?;
    person.state.outgoing_email_message().create(&person, &address, OutgoingEmailMessageKind::EmailVerification).await?;
    Ok(EmailAddress::db_to_graphql(address, false).await?)
  }
}

#[derive(GraphQLObject)]
#[graphql(description = "Email Address Verification Object: {
  id: number identifying the email address that was verified,
}")]
pub struct EmailAddressVerification {
  pub id: i32,
}

#[derive(GraphQLObject)]
#[graphql(description = "Email Address Object: {
  id: number identifying this email address,
  person_id: id of the person to whom this email address belongs,
  address: address of the person to whom this email address belongs,
  verified_at: date the email was verified if it was verified,
  keep_private: boolean pointing out whether the email is private or public,
}")]
pub struct EmailAddress {
  id: i32,
  person_id: i32,
  address: String,
  verified_at: Option<UtcDateTime>,
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
impl Showable<email_address::EmailAddress, EmailAddressFilter> for EmailAddress {
  fn sort_field_to_order_by(field: &str) -> Option<EmailAddressOrderBy> {
    match field {
      "id" => Some(EmailAddressOrderBy::Id),
      "verifiedAt" => Some(EmailAddressOrderBy::VerifiedAt),
      _ => None,
    }
  }

  fn filter_to_select(org_id: i32, f: EmailAddressFilter) -> SelectEmailAddress {
    SelectEmailAddress {
      org_id_eq: Some(org_id),
      id_in: f.ids,
      id_eq: f.id_eq,
      person_id_eq: f.person_id_eq,
      address_eq: f.address_eq,
      ..Default::default()
    }
  }

  fn select_by_id(org_id: i32, id: i32) -> SelectEmailAddress {
    SelectEmailAddress { id_eq: Some(id), org_id_eq: Some(org_id), ..Default::default() }
  }

  async fn db_to_graphql(d: email_address::EmailAddress, _with_payload: bool) -> MyResult<Self> {
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
