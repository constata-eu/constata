use super::*;

#[derive(GraphQLObject)]
#[graphql(description = "It's a manifest of all endorsement a user has")]
pub struct EndorsementManifest {
  #[graphql(description = "is always number 1")]
  pub id: i32,
  #[graphql(description = "the final text to be used as the user endorsements, if any")]
  pub text: Option<String>,
  #[graphql(description = "websites registered by the user, if any")]
  pub websites: Vec<String>,
  #[graphql(description = "data from the user's kyc endorsement, if any")]
  pub kyc: Option<KycEndorsementManifest>,
  #[graphql(description = "data from the user's telegram account, if any")]
  pub telegram: Option<TelegramEndorsementManifest>,
  #[graphql(description = "email registered by the user, if any")]
  pub email: Option<EmailEndorsementManifest>,
  #[graphql(description = "boolean pointing out whether the an email is going to be send to the student when created an issuance")]
  pub can_send_email: bool,
}

#[derive(GraphQLObject)]
#[graphql(description = "Retrieve information about user's registered telegram account, if any")]
pub struct TelegramEndorsementManifest {
  #[graphql(description = "username used in the user's telegram account, if any")]
  username: Option<String>,
  #[graphql(description = "first name of the user used in the telegram account")]
  first_name: String,
  #[graphql(description = "last name of the user used in the telegram account, if any")]
  last_name: Option<String>,
}

#[derive(GraphQLObject)]
#[graphql(description = "Retrieve information about user's registered email address, if any")]
pub struct EmailEndorsementManifest {
  #[graphql(description = "email registered by the user")]
  pub address: String,
  #[graphql(description = "boolean pointing out whether the email was registered as private or public")]
  pub keep_private: bool,
}

#[derive(GraphQLObject)]
#[graphql(description = "Retrieve information about user's kyc, if any")]
pub struct KycEndorsementManifest {
  #[graphql(description = "name of the person")]
  name: Option<String>,
  #[graphql(description = "last name of the person")]
  last_name: Option<String>,
  #[graphql(description = "number that identify the person")]
  id_number: Option<String>,
  #[graphql(description = "type of the person's id. Ej: DNI")]
  id_type: Option<String>,
  #[graphql(description = "date of birth")]
  birthdate: Option<UtcDateTime>,
  #[graphql(description = "country of birth")]
  nationality: Option<String>,
  #[graphql(description = "country in which the person currently lives")]
  country: Option<String>,
  #[graphql(description = "position that the person occupies within the company")]
  job_title: Option<String>,
  #[graphql(description = "name of the company")]
  legal_entity_name: Option<String>,
  #[graphql(description = "country in which the company has legal residence")]
  legal_entity_country: Option<String>,
  #[graphql(description = "company registration number")]
  legal_entity_registration: Option<String>,
  #[graphql(description = "company tax identification")]
  legal_entity_tax_id: Option<String>,
  #[graphql(description = "date of last update")]
  updated_at: UtcDateTime,
}

impl EndorsementManifest {
  pub async fn from_context(context: &Context) -> FieldResult<Self> {
    let person = &context.person().await;
    let text = person.endorsement_string(context.lang).await?;
    let kyc = person.kyc_endorsement().await?
      .map(|k| KycEndorsementManifest{
        name: k.attrs.name,
        last_name: k.attrs.last_name,
        id_number: k.attrs.id_number,
        id_type: k.attrs.id_type,
        birthdate: k.attrs.birthdate,
        nationality: k.attrs.nationality,
        country: k.attrs.country,
        job_title: k.attrs.job_title,
        legal_entity_name: k.attrs.legal_entity_name,
        legal_entity_country: k.attrs.legal_entity_country,
        legal_entity_registration: k.attrs.legal_entity_registration,
        legal_entity_tax_id: k.attrs.legal_entity_tax_id,
        updated_at: k.attrs.updated_at,
      });

    let websites = person.pubkey_domain_endorsement_scope()
      .state_eq(&"accepted".to_string())
      .all().await?
      .into_iter()
      .map(|o| o.attrs.domain)
      .collect();

    let telegram = person.telegram_user_scope().optional().await?
      .map(|o| TelegramEndorsementManifest{
        username: o.attrs.username,
        first_name: o.attrs.first_name,
        last_name: o.attrs.last_name,
      });

    let email = person.email_address().await?
      .map(|e| EmailEndorsementManifest {
        address: e.attrs.address,
        keep_private: e.attrs.keep_private,
      });

    let can_send_email = person.can_send_email().await?;

    Ok(Self{ id: 1, text, kyc, websites, telegram, email, can_send_email })
  }
}
