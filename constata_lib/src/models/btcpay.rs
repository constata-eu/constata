use crate::error::Error;

use rocket::{
  self,
  request::Request,
  http::Status,
  data::{self, Data, FromData, ToByteUnit},
};
use sha2::Sha256;
use hmac::{Hmac, Mac, NewMac};
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Currency {
  Eur,
  Btc,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub enum WebhookType {
  InvoiceCreated,
  InvoiceReceivedPayment,
  InvoicePaidInFull,
  InvoiceExpired,
  InvoiceSettled,
  InvoiceInvalid,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Webhook {
  pub delivery_id: String,
  pub webhook_id: String,
  pub original_delivery_id: String,
  pub is_redelivery: bool,
  #[serde(rename = "type")]
  pub kind: WebhookType,
  pub timestamp: u64,
  pub store_id: String,
  pub invoice_id: String,
}

type HmacSha256 = Hmac<Sha256>;

#[rocket::async_trait]
impl<'r> FromData<'r> for btcpay::Webhook {
  type Error = Error;

  async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {
    use rocket::data::Outcome;

    let secret = req
      .rocket()
      .state::<Site>()
      .expect("SITE not configured")
      .settings
      .btcpay
      .webhooks_secret
      .clone();

    let maybe_signature = req.headers().get_one("btcpay-sig").and_then(|x| hex::decode(&x[7..]).ok());

    match maybe_signature {
      None => return Outcome::Forward(data),
      Some(sig) => {
        let bytes = match data.open(2048.bytes()).into_bytes().await {
          Ok(read) if read.is_complete() => read.into_inner(),
          Ok(_) => return Outcome::Failure((Status::PayloadTooLarge, Error::validation("payload", "payload too large"))),
          Err(_) => return Outcome::Failure((Status::BadRequest, Error::validation("body", "Bad request, can't read body."))),
        };

        let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
          Err(_) => return Outcome::Failure((Status::BadRequest, Error::validation("body", "Unexpected error processing hmac"))),
          Ok(a) => a
        };
        mac.update(&bytes);

        match mac.verify(&sig) {
          Err(_) => Outcome::Failure((Status::BadRequest, Error::validation("bad sig", "invalid webhook signature"))),
          _ => {
            match serde_json::from_slice(&bytes) {
              Ok(webhook) => Outcome::Success(webhook),
              _ => Outcome::Failure((Status::BadRequest, Error::validation("body", "No webhook parsed"))),
            }
          }
        }
      }
    }
  }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Invoice {
  pub id: String,
  pub checkout_link: String,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct InvoiceFormCheckout {
  pub redirectURL: String,
  pub expirationMinutes: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceForm {
  pub amount: Decimal,
  pub currency: Currency,
  pub checkout: InvoiceFormCheckout
}
