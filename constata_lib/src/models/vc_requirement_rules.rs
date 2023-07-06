use super::*;

#[derive(Serialize, Deserialize)]
pub struct VcRequirementRules {
  pub acceptable_sets: Vec<RequiredSet>,
}

impl VcRequirementRules {
  pub fn vidchain_scope(&self, enabled_scopes: &[String]) -> String {
    let default = "VerifiableCredential".to_string();

    let mut maybe_desired = None;

    for required_set in &self.acceptable_sets {
      for spec in &required_set.required_set {
        for requirement in &spec.credential_spec {
          if requirement.pointer != "/type" { continue; }

          if let Filter::ArrayContains(scope) = &requirement.filter {
            if maybe_desired.map(|previous| &previous != scope).unwrap_or(false) {
              return default;
            }
            maybe_desired = Some(scope.clone());
          }
        }
      }
    }

    match maybe_desired {
      Some(desired) if enabled_scopes.contains(&desired) => desired,
      _ => default
    }
  }
}

#[derive(Serialize, Deserialize)]
pub struct RequiredSet {
  pub required_set: Vec<CredentialSpec>,
}

impl RequiredSet {
  pub fn matches(&self, credentials: &[serde_json::Value]) -> bool {
    for spec in &self.required_set {
      if !credentials.iter().any(|cred| spec.matches(cred) ) {
        return false;
      }
    }
    true
  }
}

#[derive(Serialize, Deserialize)]
pub struct CredentialSpec {
  pub credential_spec: Vec<Requirement>,
}

impl CredentialSpec {
  pub fn matches(&self, credential: &serde_json::Value) -> bool {
    for requirement in &self.credential_spec {
      let Some(value) = credential.pointer(&requirement.pointer) else { return false };
      if !requirement.filter.matches(value) { return false };
    }
    true
  }
}

#[derive(Serialize, Deserialize)]
pub struct Requirement {
  pub pointer: String,
  pub filter: Filter,
}

#[derive(Serialize, Deserialize)]
pub enum Filter {
  DateAfter(UtcDateTime),
  DateBefore(UtcDateTime),
  NumberGreaterThan(Decimal),
  NumberLesserThan(Decimal),
  StringMatches(String),
  ArrayContains(String),
}

impl Filter {
  pub fn matches(&self, value: &serde_json::Value) -> bool {
    match self {
      Self::DateAfter(date) => test(value, |v: UtcDateTime| &v > date),
      Self::DateBefore(date) => test(value, |v: UtcDateTime| &v < date),
      Self::NumberGreaterThan(number) => test(value, |v: Decimal| &v > number),
      Self::NumberLesserThan(number) => test(value, |v: Decimal| &v < number),
      Self::StringMatches(expr) => test(value, |v: String| {
        regex::Regex::new(expr).map(|re| re.is_match(&v) ).unwrap_or(false)
      }),
      Self::ArrayContains(needle) => test::<Vec<String>, _>(value, |haystack| haystack.contains(&needle)),
    }
  }
}

fn test<V: serde::de::DeserializeOwned, P: FnOnce(V) -> bool >(value: &serde_json::Value, predicate: P) -> bool {
  serde_json::from_value::<V>(value.to_owned()).map(predicate).unwrap_or(false)
}

describe! {
  dbtest!{ can_request_a_specific_credential_type (site, _c)
    let rules: VcRequirementRules = serde_json::from_str(r#"{ "acceptable_sets": [
      { "required_set": [
        { "credential_spec": [
          { "pointer":"/type", "filter":{"ArrayContains":"MedicoCredential"} }
        ]}
      ]},
      { "required_set": [
        { "credential_spec": [
          { "pointer":"/type", "filter":{"ArrayContains":"MedicoCredential"} },
          { "pointer":"/credentialStatus/statusListIndex", "filter": { "NumberGreaterThan": 40 } }
        ]}
      ]}
    ]}"#)?;
    assert_eq!(&rules.vidchain_scope(&site.settings.vidchain.enabled_scopes), "MedicoCredential");
  }

  dbtest!{ can_request_with_generic_scope (site, _c)
    let multi: VcRequirementRules = serde_json::from_str(r#"{ "acceptable_sets": [
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
    ]}"#)?;
    assert_eq!(&multi.vidchain_scope(&site.settings.vidchain.enabled_scopes), "VerifiableCredential");
  }
}
