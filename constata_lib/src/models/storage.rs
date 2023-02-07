use super::site::StorageSettings;
use crate::error::{Error, Result};

use s3::{bucket::Bucket, creds::Credentials, request_trait::ResponseData};

#[derive(Clone, Debug)]
pub struct Storage {
  pub bucket: Bucket,
  pub local: bool,
  pub files_key: Option<String>,
}

impl Storage {
  pub fn new(settings: &StorageSettings, maybe_files_key: Option<&str>) -> Result<Self> {
    let credentials: Credentials = Credentials::new(
      Some(&settings.key),
      Some(&settings.secret),
      None,
      None,
      None,
    )?;
    let bucket = Bucket::new(&settings.bucket, settings.url.parse()?, credentials)?;
    let local = settings.local.unwrap_or(false);

    let files_key = if settings.encrypt.unwrap_or(false) {
      Some(maybe_files_key.map(|k| k.to_string()).ok_or_else(|| Error::Init("Trying to encrypt storage but no key was provided".into()))?)
    } else {
      None
    };

    Ok(Self { bucket, local, files_key })
  }

  pub async fn put(&self, name: &str, payload: &[u8]) -> Result<()> {
    let ciphertext: Vec<u8>;

    let bytes: &[u8] = if let Some(k) = &self.files_key {
      ciphertext = simplestcrypt::encrypt_and_serialize(k.as_bytes(), payload)
        .map_err(|e| Error::Internal(format!("Could not encrypt: {e:?}")))?;
      &ciphertext
    } else {
      payload
    };

    if self.local {
      std::fs::write(format!("/tmp/constata-local-{}-{}", self.bucket.region, name), bytes)?;
    } else {
      let response = self.bucket.put_object(name, bytes).await?;
      if response.status_code() != 200 {
        return self.error("Writing", &response, name)
      }
    }

    Ok(())
  }

  pub async fn get(&self, name: &str) -> Result<Vec<u8>> {
    let bytes = if self.local {
      std::fs::read(format!("/tmp/constata-local-{}-{}", self.bucket.region, name))?
    } else {
      let response = self.bucket.get_object(name).await?;
      if response.status_code() != 200 {
        return self.error("Reading", &response, name)
      } else {
        response.into()
      }
    };

    if let Some(k) = &self.files_key {
      simplestcrypt::deserialize_and_decrypt(k.as_bytes(), &bytes)
        .map_err(|e| Error::Internal(format!("Could not decrypt: {e:?}")))
    } else {
      Ok(bytes)
    }
  }

  pub fn error<T>(&self, action: &str, r: &ResponseData, filename: &str) -> Result<T> {
    let content = format!("{:?}.{:?}.{}.{:?}",
      String::from_utf8_lossy(r.bytes()),
      r.status_code(),
      filename,
      self
    );
    Err(Error::third_party(&format!("{action} Storage"), &content))
  }
}
