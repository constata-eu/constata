use sha2::{Digest, Sha256};

pub fn hexdigest(bytes: &[u8]) -> String {
  let mut hasher = Sha256::new();
  hasher.update(bytes);
  format!("{:x}", hasher.finalize())
}
