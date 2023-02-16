use super::*;

#[derive(GraphQLObject)]
#[graphql(description = "The customer endorsements")]
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
#[graphql(description = "The customer telegram account endorsement")]
pub struct TelegramEndorsementManifest {
  username: Option<String>,
  first_name: String,
  last_name: Option<String>,
}

#[derive(GraphQLObject)]
#[graphql(description = "The customer telegram account endorsement")]
pub struct EmailEndorsementManifest {
  pub address: String,
  pub keep_private: bool,
}

#[derive(GraphQLObject)]
#[graphql(description = "The customer identity verification endorsement")]
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
    let person = &context.person();
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
