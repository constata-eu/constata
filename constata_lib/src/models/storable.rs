use crate::{Result, Site};

#[rocket::async_trait]
pub trait Storable: Sized {
  const PREFIX: &'static str;
  fn site(&self) -> &Site;
  fn id(&self) -> String;

  fn storage_id(&self) -> String {
    format!("{}-{}", Self::PREFIX, self.id())
  }

  async fn storage_put(&self, bytes: &[u8]) -> Result<()>{
    self.site().storage.put(&self.storage_id(), bytes).await?;
    self.storage_backup_put(bytes).await
  }

  async fn storage_backup_put(&self, bytes: &[u8]) -> Result<()> {
    self.site().storage_backup.put(&self.storage_id(), bytes).await
  }

  async fn storage_fetch(&self) -> Result<Vec<u8>> {
    self.site().storage.get(&self.storage_id()).await
  }

  async fn storage_backup_fetch(&self) -> Result<Vec<u8>> {
    self.site().storage_backup.get(&self.storage_id()).await
  }
}

macro_rules! derive_storable {
  ($struct:ident, $prefix:literal, $attr:ident) => (
    #[rocket::async_trait]
    impl Storable for $struct {
      const PREFIX: &'static str = $prefix;

      fn site(&self) -> &Site {
        &self.state
      }

      fn id(&self) -> String {
        self.attrs.$attr.to_string()
      }
    }
  );
  ($struct:ident, $prefix:literal) => (
    derive_storable!{$struct, $prefix, id}
  )
}
pub(crate) use derive_storable;
