use super::*;

use constata_lib::{signed_payload::SignedPayload, models::hasher};
use std::path::PathBuf;
use serde_with::serde_as;
use bitcoin::PublicKey;
use serde::{Deserialize, Serialize};
use simplestcrypt::deserialize_and_decrypt;
use bitcoin::{ Network, PrivateKey };

#[serde_as]
#[derive(Debug,Serialize, Deserialize)]
pub struct Config {
  public_key: PublicKey,
  #[serde_as(as = "serde_with::hex::Hex")]
  encrypted_key: Vec<u8>,
  environment: String,
}

pub struct Client {
  key: PrivateKey,
  api_url: &'static str,
  network: Network,
}

impl Client {
  pub fn new(config: Config, daily_passphrase: &str) -> ClientResult<Client> {
    let (api_url, network) = match config.environment.as_str() {
      "staging" => ("https://api-staging.constata.eu", Network::Bitcoin),
      "production" => ("https://api.constata.eu", Network::Bitcoin),
      _ => ("http://localhost:8000", Network::Regtest),
    };

    /* The javascript client generates encrypted private keys,
     * that are not compatible with rust's bincode,
     * So we need to inject the ciphertext size in the serialized encrypted key*/
    let mut fixed_encrypted_key = config.encrypted_key.clone();
    fixed_encrypted_key[16] = 68;

    let decrypted = deserialize_and_decrypt(daily_passphrase.as_bytes(), &fixed_encrypted_key)?;
    let key = PrivateKey::from_wif(&String::from_utf8(decrypted)?)?;

    Ok(Client { key, api_url, network })
  }

  pub fn from_config_file(custom_config: Option<PathBuf>, daily_passphrase: &str) -> ClientResult<Client> {
    let config_path = custom_config.to_owned().unwrap_or_else(|| "constata_conf.json".into() );
    let config: Config = check!(
      serde_json::from_str(&ex::fs::read_to_string(config_path)?),
      InvalidInput("Your config file is not a valid json")
    );
    Self::new(config, daily_passphrase)
  }

  pub fn auth_token<'a>(&'a self, nonce: i64, body: &str) -> ClientResult<String> {
    let payload = serde_json::json![{
      "path": "/graphql/",
      "method": "POST",
      "nonce": nonce,
      "body_hash": hasher::hexdigest(body.as_bytes()),
      "query_hash": None::<&str>,
    }].to_string();

    Ok(serde_json::to_string(&self.sign(payload.as_bytes()))?)
  }

  pub fn sign(&self, payload: &[u8]) -> SignedPayload {
    SignedPayload::create(payload, &self.key, self.network) 
  }

  pub fn query<R: for<'a> Deserialize<'a>, V: Serialize>(&self, vars: &V, query: &str) -> ClientResult<R> {
    let body = serde_json::to_string(&serde_json::json![{"query": query, "variables": vars }])?;
    let auth = self.auth_token(chrono::Utc::now().timestamp_millis(), &body)?;
    let endpoint = format!("{}/graphql/", self.api_url);
    let response = ureq::post(&endpoint).set("Authentication", &auth).send_string(&body)?.into_string()?;
    let gql: GqlResponse<R> = serde_json::from_str(&response)
      .map_err(|e| Error::Unexpected(format!("We could not read JSON from response \"{}\" \n\n {}", e, &response)))?;
    match gql {
      GqlResponse{ errors: errors @ Some(_), ..} => {
        Err(Error::ApiError(ApiErrorMessage::Gql(GqlResponse{ data: None, errors })))
      },
      GqlResponse{ data: Some(resource), ..} => Ok(resource),
      _ => Err(error!(Unexpected("Server replied with no data and no errors.")))
    }
  }

  pub fn simple<R, V>(&self, vars: &V, extract: &str, query: &str) -> ClientResult<R>
    where
      R: for<'a> Deserialize<'a>,
      V: Serialize
  {
      let mut response = self.query::<serde_json::Value, V>(vars, query)?;
      if let Some(value) = response.get_mut(extract) {
        Ok(serde_json::from_value(value.take())?)
      } else {
        Err(error!(Unexpected("Response had no attribute {extract}")))
      }
  }

  pub fn by_id<R, V>(&self, vars: &V, name: &str, fields: &str) -> ClientResult<R>
    where
      R: for<'a> Deserialize<'a>,
      V: Serialize
  {
      self.simple(vars, name, &format!("query($id: Int!){{ {name}(id: $id){{ {fields} }}  }}"))
  }

  pub fn mutation<R, V>(&self, vars: &V, mutation: &str, input_type: &str, fields: &str) -> ClientResult<R>
    where
      R: for<'a> Deserialize<'a>,
      V: Serialize
  {
      self.simple(vars, mutation,
        &format!("mutation($input: {input_type}!) {{ {mutation}(input: $input) {{ {fields}  }} }}")
      )
  }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GqlResponse<T> {
  pub data: Option<T>,
  pub errors: Option<Vec<GqlErrorMessage>>,
}

#[derive(Debug)]
pub enum ApiErrorMessage {
  Gql(GqlResponse<serde_json::Value>),
  Status(u16, String),
  Transport(ureq::Transport),
}

impl std::fmt::Display for ApiErrorMessage {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ApiErrorMessage::Gql(x) => {
        write!(f, "GQL, {}", serde_json::to_string_pretty(&x).unwrap_or_else(|_| format!("{:#?}", x) ))?;
      },
      ApiErrorMessage::Status(a,b) => write!(f, "Status, {} {}", a, b)?,
      ApiErrorMessage::Transport(a) => write!(f, "Transport, {:#?}", a)?,
    }
    Ok(())
  }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GqlErrorMessage {
  pub message: String,
  pub locations: Option<Vec<GqlErrorLocation>>,
  pub path: Option<Vec<String>>,
  pub extensions: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GqlErrorLocation {
  pub line: u32,
  pub column: u32,
}
