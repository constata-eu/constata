use constata_lib::{stripe, models::btcpay};
use super::*;

use rocket::{
  self,
  post,
  request::Request,
  serde::json::Json,
  State,
  http::Status,
  data::{self, Data, FromData, ToByteUnit},
};

#[post("/handle_stripe_events", data = "<webhook>")]
pub async fn handle_stripe_events(webhook: StripeWebhook, site: &State<Site>) -> JsonResult<&str> {
  site.payment().from_stripe_event(&webhook.event).await?;
  Ok(Json("OK"))
}

#[post("/handle_btcpay_webhooks", data = "<webhook>")]
pub async fn handle_btcpay_webhooks(webhook: btcpay::Webhook, site: &State<Site>) -> JsonResult<&str> {
  site.payment().from_btcpay_webhook(&webhook).await?;
  Ok(Json("OK"))
}

pub struct StripeWebhook{
  event: stripe::Event
}

#[rocket::async_trait]
impl<'r> FromData<'r> for StripeWebhook {
  type Error = Error;

  async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {
    use rocket::data::Outcome;

    let secret = req
      .rocket()
      .state::<Site>()
      .expect("SITE not configured")
      .settings
      .stripe.events_secret
      .clone();

    let maybe_signature = req.headers().get_one("stripe-signature");

    match maybe_signature {
      None => return Outcome::Forward(data),
      Some(sig) => {
        let bytes = match data.open(512000.bytes()).into_string().await {
          Ok(read) if read.is_complete() => read.into_inner(),
          Ok(_) => return Outcome::Failure((Status::PayloadTooLarge, Error::validation("payload", "payload too large"))),
          Err(_) => return Outcome::Failure((Status::BadRequest, Error::validation("body", "Bad request, can't read body."))),
        };
        match stripe::Webhook::construct_event(&bytes, &sig, &secret) {
          Ok(event) => Outcome::Success(StripeWebhook{event: event}),
          _ => Outcome::Failure((Status::BadRequest, Error::validation("body", "invalid event signature"))),
        }
      }
    }
  }
}
