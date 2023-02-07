#[cfg(any(test, feature = "test_support"))]
pub use anyhow;

pub use stripe;

#[macro_export]
macro_rules! tests {
  ($($e:tt)*) => {
    #[cfg(test)]
    mod test {
      #[allow(unused_imports)]
      use super::*;

      $($e)*
    }
  }
}

#[macro_export]
macro_rules! describe_one {
  ($($e:tt)*) => {
    #[cfg(test)]
    #[allow(unused_imports)]
    use constata_lib::{anyhow, test, test_support::*, regtest, dbtest};

    constata_lib::tests! {
      $($e)*
    }
  }
}

macro_rules! describe {
  ($($e:tt)*) => {
    tests! {
      #[allow(unused_imports)]
      use crate::{anyhow, test, test_support::*};

      $($e)*
    }
  }
}

#[cfg(any(test, feature = "test_support"))]
extern crate tokio;

#[macro_use]
#[cfg(any(test, feature = "test_support"))]
pub mod test_support;

pub use base64;
pub use base64_serde;
pub use bitcoin;
pub use bitcoin_wallet;
pub use serde;
pub use serde_derive;
pub use serde_with;
pub use thiserror;

pub mod error;
pub mod models;
pub mod signed_payload;

pub use error::{Error, Result};
pub use models::{Db, Site};

use base64_serde::base64_serde_type;
base64_serde_type!(Base64Standard, base64::STANDARD);
