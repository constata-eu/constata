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
    #[sqlx_model_hints(int4)]
    vc_requirement_id: i32,
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
    VcRequirement(vc_requirement_id),
  },
  has_many {
    VcRequest(vc_prompt_id)
  }
}

impl VcPromptHub {
  pub async fn create(&self, person: &Person, name: &str, vc_requirement: &VcRequirement) -> sqlx::Result<VcPrompt> {
    let access_token = self.state.access_token()
      .create(&person, AccessTokenKind::VcPrompt, None).await?;

    self.insert(InsertVcPrompt{
      person_id: person.attrs.id,
      org_id: person.attrs.org_id,
      name: name.to_string(),
      access_token_id: access_token.attrs.id,
      vc_requirement_id: vc_requirement.attrs.id,
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

  pub async fn get_or_create_request(&self) -> sqlx::Result<VcRequest> {
    let existing = self.vc_request_scope()
      .state_eq(VcRequestState::Pending)
      .order_by(VcRequestOrderBy::StartedAt)
      .desc(true)
      .limit(1)
      .optional().await?;

    if let Some(it) = existing {
      Ok(it)
    } else {
      self.create_request().await
    }
  }

  pub async fn create_request(&self) -> sqlx::Result<VcRequest> {
    let access_token = self.state.access_token()
      .create(&self.org().await?.admin().await?, AccessTokenKind::VcRequest, None).await?;

    for previous in self.vc_request_scope().state_eq(VcRequestState::Pending).all().await? {
      previous.finish(VcRequestState::Failed, Some("replaced_by_newer".to_string()), None, None).await?;
    }

    self.state.vc_request().insert(InsertVcRequest{
      org_id: self.attrs.org_id,
      vc_prompt_id: self.attrs.id,
      access_token_id: access_token.attrs.id
    }).save().await
  }

  pub async fn requirement_rules(&self) -> ConstataResult<VcRequirementRules> {
    Ok(serde_json::from_str(&self.vc_requirement().await?.attrs.rules)?)
  }

}

describe! {
  dbtest!{ processes_a_vc_correctly (_site, c)
    let request = process_with_rules(
      &c.alice().await.person().await,
      r#"{ "acceptable_sets": [
        { "required_set": [
          { "credential_spec": [
            { "pointer":"/issuer/id", "filter":{"StringMatches":"did:ethr:0xb97CE7b77cB8973f4132b7271E31Da0d614ACe42"} },
            { "pointer":"/issuanceDate", "filter":{"DateAfter":"2023-05-10T00:00:00.000Z"} }
          ]},
          { "credential_spec": [
            { "pointer":"/credentialSubject/Especialidad", "filter":{"StringMatches":"^Tocólog(o|a)$"} }
          ]}
        ]}
      ]}"#
    ).await?;

    assert_that!( &request.attrs, structure![ VcRequestAttrs {
      state: eq(VcRequestState::Approved),
      state_notes: eq(None),
    }]);
  }

  dbtest!{ can_submit_either_credential_set (_site, c)
    let request = process_with_rules(
      &c.alice().await.person().await,
      r#"{ "acceptable_sets": [
        { "required_set": [
          { "credential_spec": [
            { "pointer":"/type", "filter":{"ArrayContains":"MedicoCredential"} }
          ]}
        ]},
        { "required_set": [
          { "credential_spec": [
            { "pointer":"/type", "filter":{"ArrayContains":"StudentCredential"} }
          ]}
        ]}
      ]}"#
    ).await?;

    assert_that!( &request.attrs, structure![ VcRequestAttrs {
      state: eq(VcRequestState::Approved),
      state_notes: eq(None),
    }]);
  }

  dbtest!{ is_missing_a_credential (_site, c)
    let request = process_with_rules(
      &c.alice().await.person().await,
      r#"{ "acceptable_sets": [
        { "required_set": [
          { "credential_spec": [
            { "pointer":"/issuer/id", "filter":{"StringMatches":"did:ethr:0xb97CE7b77cB8973f4132b7271E31Da0d614ACe42"} }
          ]},
          { "credential_spec": [
            { "pointer":"/credentialSubject/Especialidad", "filter":{"StringMatches":"^UNMATCHED$"} }
          ]}
        ]}
      ]}"#
    ).await?;

    assert_that!( &request.attrs, structure![ VcRequestAttrs {
      state: eq(VcRequestState::Rejected),
      state_notes: eq(Some("presentation_does_not_meet_requirements".to_string())),
    }]);
  }

  dbtest!{ can_filter_numeric_values (_site, c)
    let person = c.alice().await.person().await;
    let approved = process_with_rules(
      &person,
      r#"{ "acceptable_sets": [
        { "required_set": [
          { "credential_spec": [
            { "pointer":"/credentialStatus/statusListIndex", "filter": { "NumberGreaterThan": 20 } },
            { "pointer":"/credentialStatus/statusListIndex", "filter": { "NumberLesserThan": 40 } }
          ]}
        ]}
      ]}"#
    ).await?;
    assert_eq!(approved.attrs.state, VcRequestState::Approved);

    let too_high = process_with_rules(
      &person,
      r#"{ "acceptable_sets": [
        { "required_set": [
          { "credential_spec": [
            { "pointer":"/credentialStatus/statusListIndex", "filter": { "NumberLesserThan": 10 } }
          ]}
        ]}
      ]}"#
    ).await?;

    assert_eq!(too_high.attrs.state, VcRequestState::Rejected);

    let too_low = process_with_rules(
      &person,
      r#"{ "acceptable_sets": [
        { "required_set": [
          { "credential_spec": [
            { "pointer":"/credentialStatus/statusListIndex", "filter": { "NumberGreaterThan": 40 } }
          ]}
        ]}
      ]}"#
    ).await?;

    assert_eq!(too_low.attrs.state, VcRequestState::Rejected);
  }

  async fn process_with_rules(person: &Person, rules: &str) -> anyhow::Result<VcRequest> {
    let org = person.org().await?;

    let requirement = person.state.vc_requirement().insert(InsertVcRequirement{
      org_id: org.attrs.id,
      name: "Testing rules".to_string(),
      rules: rules.to_string(),
    }).save().await?;
    
    let prompt = person.state.vc_prompt().create(&person, "museum entrance 1", &requirement).await?;
    let mut request = prompt.create_request().await?;
    request = request.resolve_with_vidchain_jwt(
      "GfkE9rv6MokiPX431swk8Ft7PRag7idzIQQNDgb9pPE.qt49pQ8JmQMp9r1KfV5K0KRUuxBcltTfPGn9xQ7QnXQ",
      read_to_string("vc_requests/wrapped_verifiable_presentation_response.json")
    ).await?;

    Ok(request)
  }
}

