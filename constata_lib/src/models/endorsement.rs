use juniper::GraphQLObject;
use crate::{ ConstataResult, models::{ Person, UtcDateTime, }, };

#[derive(PartialEq, Clone, Debug, serde::Serialize)]
#[serde(tag = "type")]
pub enum Endorsement {
  Website { url: String },
  EmailAddress { address: String, keep_private: bool },
  Kyc { attrs: super::kyc_endorsement::KycEndorsementAttrs, },
}

pub mod for_api {
  use super::*;
  #[derive(Debug, Clone, GraphQLObject, serde::Serialize, serde::Deserialize)]
  #[graphql(description = "A Person may have many endorsements from Constata, such as their identity, owning an email account, or being the manager of a website. This manifest gathers all those endorsements. You can only query your own manifest for now.")]
  pub struct EndorsementManifest {
    #[graphql(description = "You'll always get number 1.")]
    pub id: i32,
    #[graphql(description = "the final text to be used as the user endorsements, if any")]
    pub text: Option<String>,
    #[graphql(description = "websites registered by the user, if any")]
    pub websites: Vec<String>,
    #[graphql(description = "data from the user's kyc endorsement, if any")]
    pub kyc: Option<KycEndorsementManifest>,
    #[graphql(description = "email registered by the user, if any")]
    pub email: Option<EmailEndorsementManifest>,
    #[graphql(description = "boolean pointing out whether the an email is going to be send to the student when created an issuance")]
    pub can_send_email: bool,
  }

  #[derive(Debug, Clone, GraphQLObject, serde::Serialize, serde::Deserialize)]
  #[graphql(description = "Your email account, as verified by Constata's email robot.")]
  pub struct EmailEndorsementManifest {
    pub address: String,
    #[graphql(description = "Whether you told us to use this email as part of your public endorsements or not.")]
    pub keep_private: bool,
  }

  #[derive(Debug, Clone, GraphQLObject, serde::Serialize, serde::Deserialize)]
  #[graphql(description = "The personal and company details you sent Constata for verification and to include in all your signed certificates. Keep in mind all fields are optional, this data is protected by data protection laws such as GDPR. https://api.constata.eu/terms_acceptance/show/#privacy_policies ")]
  pub struct KycEndorsementManifest {
    #[graphql(description = "Your first names")]
    name: Option<String>,
    #[graphql(description = "Your last names")]
    last_name: Option<String>,
    #[graphql(description = "Government or otherwise officially issued ID number")]
    id_number: Option<String>,
    #[graphql(description = "Type of the officially issued id. Ej: DNI")]
    id_type: Option<String>,
    #[graphql(description = "Date of birth")]
    birthdate: Option<UtcDateTime>,
    #[graphql(description = "Country of birth")]
    nationality: Option<String>,
    #[graphql(description = "Country you currently live in.")]
    country: Option<String>,
    #[graphql(description = "Your role, title or position in your company, if any.")]
    job_title: Option<String>,
    #[graphql(description = "Name of the company")]
    legal_entity_name: Option<String>,
    #[graphql(description = "Country where the company is based on, or where it has its HQ.")]
    legal_entity_country: Option<String>,
    #[graphql(description = "Company registration number in the required public registries, if any.")]
    legal_entity_registration: Option<String>,
    #[graphql(description = "Company tax identification number")]
    legal_entity_tax_id: Option<String>,
    #[graphql(description = "Date of last update to this data.")]
    updated_at: UtcDateTime,
  }

  impl EndorsementManifest {
    pub async fn from_person(person: &super::Person, lang: i18n::Lang) -> ConstataResult<EndorsementManifest> {
      let text = person.endorsement_string(lang, true).await?;
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

      let email = person.email_address().await?
        .map(|e| EmailEndorsementManifest {
          address: e.attrs.address,
          keep_private: e.attrs.keep_private,
        });

      let can_send_email = person.can_send_email().await?;

      Ok(Self{ id: 1, text, kyc, websites, email, can_send_email })
    }
  }
}

