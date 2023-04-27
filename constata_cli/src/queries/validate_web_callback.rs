use super::*;
use constata_lib::models::web_callback::WebCallbackContent;
use bitcoin::network::constants::Network;

#[derive(serde::Serialize, clap::Args)]
pub struct ValidateWebCallback {
  #[arg(help="Full HTTP body of the web callback (it's JSON)")]
  pub content: Option<String>,
  #[arg(short, long, help="A file containing the web callback you want to validate.")]
  pub input_file: Option<PathBuf>,
  #[arg(short, long, default_value="production", help="The environment from where you got the callback.")]
  pub environment: Environment,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, clap::ValueEnum)]
pub enum Environment {
  Production,
  Staging,
  Development,
}

impl ValidateWebCallback {
  pub fn run(mut self, _client: &Client) -> ClientResult<WebCallbackContent> {
    if let Some(ref path) = self.input_file {
      self.content = Some(ex::fs::read_to_string(path)?);
    }

    let Some(content) = self.content else {
      return Err(error!(InvalidInput("You need to provide a web callback to validate")));
    };

    let (expected, network) = match self.environment {
      Environment::Development => ("bcrt1qsj2h8ernt4amc674l60vu925flvn57ff9lyry2", Network::Regtest),
      Environment::Staging => ("tb1qurghvhp8g6he5hsv0en6n59rextfw8kw0wxyun", Network::Testnet),
      Environment::Production => ("bc1qw3ca5pgepg6hqqle2eq8qakejl5wdafs7up0jd", Network::Bitcoin),
    };

    let signed: SignedPayload = serde_json::from_str(&content)?;
    let good_signature = check!(signed.signed_ok(), Unexpected("Could not validate signature"));
    let signer = check!(signed.signer_as_p2wpkh(network), Unexpected("Could not obtain signer address"));

    if good_signature && signer == expected {
      Ok(serde_json::from_slice(&signed.payload)?)
    } else {
      Err(error!(InvalidInput("The web callback signature was wrong, expected signer to be {} but was {}", expected, signer)))
    }
  }
}
