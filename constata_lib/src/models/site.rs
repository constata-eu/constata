use super::AuditLog;
use crate::{Error, ConstataResult};
use bitcoin::network::constants::Network;
use stripe::Client;
use rocket::Config;
use serde::{Deserialize, Serialize};
use sqlx::{ConnectOptions, postgres::{PgPoolOptions, PgConnectOptions}};
pub use sqlx_models_orm::Db;
use super::storage::Storage;
use chrono::Duration;
use bitcoin_wallet::account::{Account, AccountAddressType, MasterAccount, Unlocker};
use bitcoin::{ util::bip32::ExtendedPubKey, Address, PrivateKey };
use std::str::FromStr;
use std::io::{stdin, Read};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct SiteSettings {
  pub database_uri: String,
  pub network: Network,
  pub url: String,
  pub wallet_encrypted_hex: String,
  pub wallet_xpub: String,
  pub wallet_address: Address,
  pub bitcoin_rpc_uri: String,
  pub bitcoin_rpc_user: String,
  pub bitcoin_rpc_pass: String,
  pub api_key: String,
  pub minimum_bulletin_interval: i64,
  pub bump_interval: Option<i64>,
  pub delete_old_parked_interval: Option<i64>,
  pub pubkey_domain_endorsement_retry_wait: i64,
  pub audit_log_path: String,
  pub audit_log_max_size: u64,
  pub payment_success_redirect: String,
  pub payment_error_redirect: String,
  pub stripe: StripeSettings,
  pub btcpay: BtcPaySettings,
  pub worker_wait_time: Option<u64>,
  pub testing_database: Option<bool>,
  pub testing_storage: Option<bool>,
  pub storage: StorageSettings,
  pub storage_backup: StorageSettings,
  pub vidchain: VidchainSettings,
  pub email_bot_sender: Option<String>,
  pub customer_care_email: Option<String>,
  pub recaptcha_threshold: Option<f64>,
  pub default_logo_url: Option<String>,
}

impl SiteSettings {
  pub fn default() -> ConstataResult<Self> {
    Ok(Config::figment().extract::<Self>()?)
  }

  pub fn recaptcha_threshold(&self) -> f64 {
    self.recaptcha_threshold.unwrap_or(0.2)
  }

  pub fn email_bot_sender(&self) -> &str {
    self.email_bot_sender.as_deref().unwrap_or("Constata EU <ace@constata.eu>")
  }

  pub fn customer_care_email(&self) -> &str {
    self.customer_care_email.as_deref().unwrap_or("Ayuda de Constata <hola@constata.eu>")
  }

  pub fn default_logo_url(&self) -> &str {
    self.default_logo_url.as_deref().unwrap_or("https://constata.eu/assets/images/logo.png")
  }

  pub fn minimum_bulletin_interval(&self) -> Duration {
    Duration::minutes(self.minimum_bulletin_interval)
  }

  pub fn bump_interval(&self) -> Duration {
    Duration::minutes(self.bump_interval.unwrap_or(60))
  }

  pub fn delete_old_parked_interval(&self) -> Duration {
    Duration::days(self.delete_old_parked_interval.unwrap_or(40))
  }

  pub fn environment(&self) -> Environment {
    self.network.into()
  }
}

#[derive(Clone)]
pub struct Site {
  pub settings: Box<SiteSettings>,
  pub db: Db,
  pub audit_log: AuditLog,
  pub stripe_client: Box<stripe::Client>,
  pub storage: Box<Storage>,
  pub storage_backup: Box<Storage>,
  pub password: Option<String>,
}

impl core::fmt::Debug for Site {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    f.write_str("Site")
  }
}

impl Site {
  pub async fn from_stdin_password() -> ConstataResult<Self> {
    let mut password = String::new();
    stdin().read_to_string(&mut password)?;
    password.pop(); // Remove newline
    Self::from_settings_and_password(SiteSettings::default()?, Some(&password)).await
  }

  pub async fn default() -> ConstataResult<Self> {
    Self::from_settings_and_password(SiteSettings::default()?, None).await
  }

  pub async fn default_with_keyring(password: &str) -> ConstataResult<Self> {
    Self::from_settings_and_password(SiteSettings::default()?, Some(password)).await
  }

  pub async fn from_settings(settings: SiteSettings) -> ConstataResult<Self> {
    Self::from_settings_and_password(settings, None).await
  }

  pub async fn from_settings_and_password(settings: SiteSettings, password: Option<&str>) -> ConstataResult<Self> {
    let mut options = PgConnectOptions::from_str(&settings.database_uri)?;
    options.disable_statement_logging();
    let pool_options = PgPoolOptions::new().max_connections(5);
    let pool = pool_options.connect_with(options).await?;
    let db = Db{ pool, transaction: None };
    let audit_log = AuditLog::new(settings.audit_log_path.clone(), settings.audit_log_max_size);
    let stripe_client = Client::new(&settings.stripe.secret_key);

    let keyring = password.map(|p| Keyring::from_settings_and_password(&settings, p)).transpose()?;
    let files_key = keyring.as_ref().map(|k| &*k.files_key );
    let storage = Storage::new(&settings.storage, files_key)?;
    let storage_backup = Storage::new(&settings.storage_backup, files_key)?;

    Ok(Self{
      settings: Box::new(settings),
      db,
      audit_log,
      stripe_client: Box::new(stripe_client),
      storage: Box::new(storage),
      storage_backup: Box::new(storage_backup),
      password: password.map(|p| p.to_string())
    })
  }

  pub fn keyring(&self) -> ConstataResult<Option<Keyring>> {
    self.password.as_ref().map(|p| Keyring::from_settings_and_password(&self.settings, &p)).transpose()
  }
}

pub struct Keyring {
  pub private_key: PrivateKey,
  pub master_account: MasterAccount,
  pub address: Address,
  pub unlocker: Unlocker,
  pub files_key: String,
}

impl Keyring {
  fn from_settings_and_password(settings: &SiteSettings, password: &str) -> ConstataResult<Self> {
    let pubkey = ExtendedPubKey {
      network: settings.network,
      ..ExtendedPubKey::from_str(&settings.wallet_xpub).map_err(|_| Error::Init("Invalid wallet_xpub".to_string()))?
    };

    let mut master_account = MasterAccount::from_encrypted(
      &hex::decode(&settings.wallet_encrypted_hex).map_err(|_| Error::Init("Invalid wallet_encrypted_hex".to_string()))?,
      pubkey,
      0
    );
    let mut unlocker = Unlocker::new_for_master(&master_account, password)?;
    master_account.add_account(Account::new(&mut unlocker, AccountAddressType::P2WPKH, 1, 0, 10).unwrap());
    let private_key = unlocker.unlock(AccountAddressType::P2WPKH, 1, 0, 1, None)?;

    let address = master_account
      .get((1, 0))
      .unwrap()
      .get_key(1)
      .unwrap()
      .address
      .clone();

    let mut files_key = super::hasher::hexdigest(&private_key.clone().to_bytes());
    files_key.truncate(32); // Our encription lib only supports keys up to 32 bytes.

    Ok(Self{ private_key, master_account, address, unlocker, files_key })
  }
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct StripeSettings {
  pub checkout_domain: String,
  pub secret_key: String,
  pub public_key: String,
  pub events_secret: String,
  pub token_product_code: String,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct BtcPaySettings {
  pub base_url: String,
  pub store_id: String,
  pub api_key: String,
  pub webhooks_secret: String,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct StorageSettings {
  pub key: String,
  pub url: String,
  pub secret: String,
  pub bucket: String,
  pub local: Option<bool>,
  pub encrypt: Option<bool>,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct VidchainSettings {
  pub host: String,
  pub redirect_uri: String,
  pub client_id: String,
  pub client_secret: String,
  pub expected_kid: String,
  pub rsa_pk_n: String,
  pub did_ethr: String,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, juniper::GraphQLEnum)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
  Production,
  Staging,
  Development
}

impl From<Network> for Environment {
  fn from(network: Network) -> Self {
    match network {
      Network::Bitcoin => Environment::Production,
      Network::Testnet | Network::Signet => Environment::Staging,
      Network::Regtest => Environment::Development,
    }
  }
}
