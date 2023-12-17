#[macro_export]
macro_rules! pub_mods {
  [ $($mod:ident::$child:ident;)+ ] => (
    $(
      pub mod $mod;
      pub use $mod::$child;
    )+
  );
  [ $($mod:ident::{$($child:ident),+};)+] => (
    $(
      pub mod $mod;
      pub use $mod::{$($child,)+};
    )+
  );
  [ $($mod:ident;)+ ] => (
    $(
      pub mod $mod;
      pub use $mod::*;
    )+
  );
}

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
    use constata_lib::{anyhow, test, requires_setting, test_support::*, regtest, dbtest};

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

#[cfg(any(test, feature = "test_support"))]
pub use anyhow;

pub use stripe;
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
pub mod graphql;

pub use error::{Error, ConstataResult};
pub use models::{Db, Site};

use base64_serde::base64_serde_type;
base64_serde_type!(pub Base64Standard, base64::STANDARD);

i18n::make_static_renderer!(RENDERER, "$CARGO_MANIFEST_DIR/templates");

pub mod prelude {
  pub use crate::{
    models::{Db, Site, hasher::hexdigest},
    error::{Error, ConstataResult },
    Base64Standard,
    signed_payload::{self, SignedPayload},
  };
  pub use serde::{Serialize, Deserialize};
  pub use chrono::{DateTime, Duration, Utc, Datelike, TimeZone};
  pub type UtcDateTime = DateTime<Utc>;
  pub use serde_with::{serde_as, DisplayFromStr};
  pub use rust_decimal::Decimal;
  pub use rust_decimal_macros::dec;
}
