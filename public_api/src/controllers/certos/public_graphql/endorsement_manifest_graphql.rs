use super::*;

#[derive(GraphQLObject)]
#[graphql(description = "Endorsement Manifest Object: {
  id: is always number 1,
  text: the final text to be used as the user endorsements, if any,
  websites: websites registered by the user, if any,
  kyc: data from the user's kyc endorsement, if any,
  telegram: data from the user's telegram account, if any,
  email: email registered by the user, if any,
  can_send_email: boolean pointing out whether the an email is going to be send to the student when created an issuance,
}")]
pub struct EndorsementManifest {
  pub id: i32,
  pub text: Option<String>,
  pub websites: Vec<String>,
  pub kyc: Option<KycEndorsementManifest>,
  pub telegram: Option<TelegramEndorsementManifest>,
  pub email: Option<EmailEndorsementManifest>,
  pub can_send_email: bool,
}

#[derive(GraphQLObject)]
#[graphql(description = "Telegram Endorsement Manifest Object: {
  username: username used in the user's telegram account, if any,
  first_name: first name of the user used in the telegram account,
  last_name: last name of the user used in the telegram account, if any,
}")]
pub struct TelegramEndorsementManifest {
  username: Option<String>,
  first_name: String,
  last_name: Option<String>,
}

#[derive(GraphQLObject)]
#[graphql(description = "Email Endorsement Manifest Object: {
  address: email registered by the user,
  keep_private: boolean pointing out whether the email was registered as private or public,
}")]
pub struct EmailEndorsementManifest {
  pub address: String,
  pub keep_private: bool,
}

#[derive(GraphQLObject)]
#[graphql(description = "Kyc Endorsement Manifest Object: {
  name: name of the person,
  last_name: last name of the person,
  id_number: number that identify the person,
  id_type: type of the person's id. Ej: DNI,
  birthdate: date of birth,
  nationality: country of birth,
  country: country in which the person currently lives,
  job_title: position that the person occupies within the company,
  legal_entity_name: name of the company,
  legal_entity_country: country in which the company has legal residence,
  legal_entity_registration: company registration number,
  legal_entity_tax_id: company tax identification,
  updated_at: date of last update,
}")]
pub struct KycEndorsementManifest {
  name: Option<String>,
  last_name: Option<String>,
  id_number: Option<String>,
  id_type: Option<String>,
  birthdate: Option<UtcDateTime>,
  nationality: Option<String>,
  country: Option<String>,
  job_title: Option<String>,
  legal_entity_name: Option<String>,
  legal_entity_country: Option<String>,
  legal_entity_registration: Option<String>,
  legal_entity_tax_id: Option<String>,
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
