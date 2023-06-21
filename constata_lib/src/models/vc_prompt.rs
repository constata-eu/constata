use super::*;

model!{
  state: Site,
  table: vc_prompts,
  struct VcPrompt {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    org_id: i32,
    #[sqlx_model_hints(int4)]
    person_id: i32,
    #[sqlx_model_hints(varchar)]
    name: String,
    #[sqlx_model_hints(int4)]
    access_token_id: i32,
    #[sqlx_model_hints(text)]
    rules: String,
    #[sqlx_model_hints(timestamptz, default)]
    created_at: UtcDateTime,
    #[sqlx_model_hints(timestamptz, default)]
    archived_at: Option<UtcDateTime>,
    #[sqlx_model_hints(int4, default)]
    deletion_id: Option<i32>,
  },
  belongs_to {
    Org(org_id),
    Person(person_id),
    AccessToken(access_token_id),
    OrgDeletion(deletion_id),
  },
  has_many {
    VcRequest(vc_prompt_id)
  }
}

impl VcPromptHub {
  pub async fn create(&self, person: &Person, name: &str, rules: &str) -> sqlx::Result<VcPrompt> {
    let access_token = self.state.access_token()
      .create(&person, AccessTokenKind::VcPrompt, None).await?;

    self.insert(InsertVcPrompt{
      person_id: person.attrs.id,
      org_id: person.attrs.org_id,
      name: name.to_string(),
      access_token_id: access_token.attrs.id,
      rules: rules.to_string(),
    }).save().await
  }
}

impl VcPrompt {
  pub async fn archive(self) -> sqlx::Result<Self> {
    self.update().archived_at(Some(Utc::now())).save().await
  }

  pub async fn full_url(&self) -> ConstataResult<String> {
    Ok(format!("{}/#/vc_prompt_kiosk/{}", &self.state.settings.url, self.access_token().await?.attrs.token))
  }

  pub async fn create_request(&self) -> sqlx::Result<VcRequest> {
    let access_token = self.state.access_token()
      .create(&self.org().await?.admin().await?, AccessTokenKind::VcRequest, None).await?;

    self.state.vc_request().insert(InsertVcRequest{
      org_id: self.attrs.org_id,
      vc_prompt_id: self.attrs.id,
      access_token_id: access_token.attrs.id
    }).save().await
  }
}

describe! {
  dbtest!{ creates_prompt_and_receives_correct_vc (site, _c)
    use ssi::jwk::JWK;
    use serde_json::json;
    use ssi::ldp::{Check, LinkedDataProofOptions};

    let conf = site.settings.vidchain;
    let key: JWK = serde_json::from_value(serde_json::json!({
      "kty": "RSA",
      "n": conf.rsa_pk_n,
      "e": "AQAB",
      "alg": "RS256",
      "kid": conf.expected_kid,
    })).unwrap();

    let response: serde_json::Value = serde_json::from_str(&read_to_string("vc_requests/wrapped_verifiable_presentation_response.json"))?;
    let presentation = response["id_token"].as_str().unwrap();
    let mut claims: serde_json::Value = ssi::jwt::decode_verify(presentation, &key).unwrap();

    assert_eq!(&claims["aud"], &json!{["constata"]});
    assert_eq!(&claims["did"], &json!{"did:ethr:0x22F57B008a1321F71d31d60b16EC50af949102bc"});
    *claims.get_mut("vp").unwrap().get_mut("@context").unwrap() = json!("https://www.w3.org/2018/credentials/v1");
    /*
    *claims.get_mut("vp").unwrap()
      .get_mut("proof").unwrap()
      .get_mut("verificationMethod").unwrap() = json!("did:ethr:0x22F57B008a1321F71d31d60b16EC50af949102bc#controller");
      */

    //std::fs::write("claims.json", &serde_json::to_string_pretty(&claims)?);

    let vp = ssi::vc::Presentation::from_json(&serde_json::to_string(&claims["vp"].take()).unwrap()).unwrap();

    let mut context_loader = ssi::jsonld::ContextLoader::default();
    //let proof: ssi::ldp::Proof = serde_json::from_value(claims["vp"]["proof"].clone())?;
    //vp.add_proof(proof);
    vp.validate().unwrap();

    /*
    let verify_options = LinkedDataProofOptions {
      checks: Some(vec![Check::Proof, Check::Status]),
      ..Default::default()
    };
    */
    let verify_options = LinkedDataProofOptions::default();
    let verification_result = vp.verify(Some(verify_options), &did_ethr::DIDEthr, &mut context_loader).await;
    dbg!(verification_result);

    // Assert it's not expired by credential_status.
    // Assert by issuance_date.
    // Assert by expiration_date.
    // Assert by type. "MedicoCredential"
    // Assert by issuer did:ethr:0xb97CE7b77cB8973f4132b7271E31Da0d614ACe42
    //
    //assert_that!(vp.verifiable_credential, 

    // Always: credential_status.
    // Any Doctor (Is a MedicoCredential)
    // Members of Colegio de médicos
    // Old members of colegio de médicos.

    //let key = RS256PublicKey::from_components(Base64UrlSafe::decode(&conf.rsa_pk_n), Base64UrlSafe::decode(&conf.rsa_pk_e));
    /*
    let value: serde_json::Value = serde_json::from_str(&read_to_string("vc_requests/correct_vc_jwt.json"))?;
    let token: &str = value["id_token"].as_str().unwrap();
    std::fs::write("token.jwt", &token);
    dbg!(&token);
    */
    //let unverified = String::from_utf8( base64::URL_SAFE::decode(token.split(".").skip(1).next().unwrap())?)?;
    // std::fs::write("unverified.json", &unverified);
    //dbg!(unverified);


    todo!("Fail here");


    /*
    use hmac::{Hmac, Mac, NewMac};
    use jwt::VerifyWithKey;
    use sha2::Sha256;
    use std::collections::BTreeMap;

    let key: Hmac<Sha256> = Hmac::new_from_slice(b"RJHFqjyQsPS2bm9Tt5snFzyqLy").unwrap();
    let value: serde_json::Value = serde_json::from_str(&read_to_string("vc_requests/correct_vc_jwt.json"))?;
    let claims: BTreeMap<String, String> = value["access_token"].as_str().unwrap().verify_with_key(&key)?;
    dbg!(claims);
    */
    //assert_eq!(claims["sub"], "someone");
    

    /*
    let alice = c.alice().await;
    let prompt = site.vc_prompt().create(&alice.person().await, "museum entrance 1", "VerifiableCreated").await?;
    let request = prompt.create_request().await?;

    let accepted = request.resolve_with_vidchain_jwt(
      "GfkE9rv6MokiPX431swk8Ft7PRag7idzIQQNDgb9pPE.qt49pQ8JmQMp9r1KfV5K0KRUuxBcltTfPGn9xQ7QnXQ",
      read_to_string("vc_requests/correct_vc_jwt.json")
    ).await?;

    assert_eq!(accepted.attrs.state, VcRequestState::Approved);
    */
  }
}

