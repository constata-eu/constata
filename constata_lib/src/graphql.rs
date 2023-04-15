/// This module defines Constata's custom GraphQL scalar, and all our custom scalar types, as required by Juniper.
/// This makes it more ergonomic to use our internal types in the API. 

use bitcoin::{ util::misc::MessageSignature, Address };
use serde::{Deserialize, Serialize, de::Deserializer, de};
use juniper::{ graphql_scalar, parser::ScalarToken, InputValue, Value, ParseScalarResult, ParseScalarValue, ScalarValue};

#[derive(Clone, Debug, PartialEq, ScalarValue, Serialize)]
#[serde(untagged)]
pub enum GqlScalar {
    #[value(as_float, as_int)]
    Int(i32),
    Long(i64),
    #[value(as_float)]
    Float(f64),
    #[value(as_str, as_string, into_string)]
    String(String),
    #[value(as_bool)]
    Boolean(bool),
}

impl<'de> Deserialize<'de> for GqlScalar {
  fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
    struct Visitor;

    impl<'de> de::Visitor<'de> for Visitor {
      type Value = GqlScalar;

      fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("a valid input value")
      }

      fn visit_bool<E: de::Error>(self, b: bool) -> Result<Self::Value, E> {
        Ok(GqlScalar::Boolean(b))
      }

      fn visit_i32<E: de::Error>(self, n: i32) -> Result<Self::Value, E> {
        Ok(GqlScalar::Int(n))
      }

      fn visit_i64<E: de::Error>(self, b: i64) -> Result<Self::Value, E> {
        if b <= i64::from(i32::MAX) {
          self.visit_i32(b.try_into().unwrap())
        } else {
          Ok(GqlScalar::Long(b))
        }
      }

      fn visit_u32<E: de::Error>(self, n: u32) -> Result<Self::Value, E> {
        if n <= i32::MAX as u32 {
          self.visit_i32(n.try_into().unwrap())
        } else {
          self.visit_u64(n.into())
        }
      }

      fn visit_u64<E: de::Error>(self, n: u64) -> Result<Self::Value, E> {
        if n <= i64::MAX as u64 {
          self.visit_i64(n.try_into().unwrap())
        } else {
          // Browser's `JSON.stringify()` serializes all numbers
          // having no fractional part as integers (no decimal point),
          // so we must parse large integers as floating point,
          // otherwise we would error on transferring large floating
          // point numbers.
          // TODO: Use `FloatToInt` conversion once stabilized:
          //       https://github.com/rust-lang/rust/issues/67057
          Ok(GqlScalar::Float(n as f64))
        }
      }

      fn visit_f64<E: de::Error>(self, f: f64) -> Result<Self::Value, E> {
        Ok(GqlScalar::Float(f))
      }

      fn visit_str<E: de::Error>(self, s: &str) -> Result<Self::Value, E> {
        self.visit_string(s.into())
      }

      fn visit_string<E: de::Error>(self, s: String) -> Result<Self::Value, E> {
        Ok(GqlScalar::String(s))
      }
    }

    de.deserialize_any(Visitor)
  }
}

#[graphql_scalar(with = bytes, scalar = GqlScalar)]
pub type Bytes = Vec<u8>;

mod bytes {
  use super::*;

  pub(super) fn to_output(v: &Bytes) -> Value<GqlScalar> {
    Value::scalar(base64::encode(&v))
  }

  pub(super) fn from_input(v: &InputValue<GqlScalar>) -> Result<Bytes, String> {
    let string = v.as_string_value().ok_or_else(|| "value was not serializable".to_string())?;
    base64::decode(string)
      .map_err(|_| format!("Expected a base64 encoded string, found: {v}"))
  }

  pub(super) fn parse_token(value: ScalarToken<'_>) -> ParseScalarResult<GqlScalar> {
    <String as ParseScalarValue<GqlScalar>>::from_str(value)
  }
}

#[graphql_scalar(with = msg_sig, scalar = GqlScalar)]
pub type MsgSig = MessageSignature;

mod msg_sig {
  use super::*;

  pub(super) fn to_output(v: &MsgSig) -> Value<GqlScalar> {
    Value::scalar(v.to_string())
  }

  pub(super) fn from_input(v: &InputValue<GqlScalar>) -> Result<MsgSig, String> {
    let string = v.as_string_value().ok_or_else(|| "value was not serializable".to_string())?;
    <MsgSig as std::str::FromStr>::from_str(string).map_err(|_| format!("Expected an encoded message signature, found: {v}"))
  }

  pub(super) fn parse_token(value: ScalarToken<'_>) -> ParseScalarResult<GqlScalar> {
    <String as ParseScalarValue<GqlScalar>>::from_str(value)
  }
}

#[graphql_scalar(with = addr, scalar = GqlScalar)]
pub type Addr = Address;

mod addr {
  use super::*;

  pub(super) fn to_output(v: &Addr) -> Value<GqlScalar> {
    Value::scalar(v.to_string())
  }

  pub(super) fn from_input(v: &InputValue<GqlScalar>) -> Result<Addr, String> {
    let string = v.as_string_value().ok_or_else(|| "value was not serializable".to_string())?;
    <Addr as std::str::FromStr>::from_str(string).map_err(|_| format!("Expected an encoded address, found: {v}"))
  }

  pub(super) fn parse_token(value: ScalarToken<'_>) -> ParseScalarResult<GqlScalar> {
    <String as ParseScalarValue<GqlScalar>>::from_str(value)
  }
}
