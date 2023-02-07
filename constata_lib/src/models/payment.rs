use crate::error::*;
use super::*;

model!{
  state: Site,
  table: payments,
  struct Payment {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    org_id: i32,
    #[sqlx_model_hints(timestamptz)]
    created_at: UtcDateTime,
    #[sqlx_model_hints(decimal)]
    amount: Decimal,
    #[sqlx_model_hints(decimal)]
    tokens: Decimal,
    #[sqlx_model_hints(decimal)]
    fees: Decimal,
    #[sqlx_model_hints(payment_source)]
    payment_source: PaymentSource,
    clearing_data: String,
    #[sqlx_model_hints(int4)]
    invoice_id: Option<i32>,
  }
}

impl InsertPaymentHub {
  pub async fn save_and_trigger_updates(self) -> Result<Payment> {
    let payment = self.save().await?;

    if let Some(id) = payment.attrs.invoice_id {
      payment.state.db.execute(sqlx::query!(
        "UPDATE invoices SET paid = true, payment_id = $2 WHERE id = $1", 
        id,
        payment.id(),
      )).await?;
    }

    payment.state.org()
      .find(payment.org_id()).await?
      .account_state().await?
      .fund_all_documents().await?;

    Ok(payment)
  }
}

impl PaymentHub {
  pub async fn from_btcpay_webhook(&self, webhook: &btcpay::Webhook) -> Result<Option<Payment>> {
    if webhook.kind != btcpay::WebhookType::InvoiceSettled {
      return Ok(None)
    }

    let maybe_invoice = self.state.invoice()
      .select()
      .external_id_eq(&webhook.invoice_id)
      .payment_source_eq(&PaymentSource::BtcPay)
      .optional()
      .await?;

    if maybe_invoice.as_ref().and_then(|i| i.attrs.payment_id ).is_some() {
      return Ok(None)
    }

    if let Some(mut invoice) = maybe_invoice {
      Ok(Some(invoice.make_payment(None).await?))
    } else {
      Ok(None)
    }
  }

  pub async fn from_invoice(&self, invoice_id: i32) -> Result<Option<Payment>> {
    let maybe_invoice = self.state.invoice()
      .select()
      .id_eq(&invoice_id)
      .payment_id_is_set(false)
      .optional()
      .await?;

    if let Some(mut invoice) = maybe_invoice {
      Ok(Some(invoice.make_payment(None).await?))
    } else {
      Ok(None)
    }
  }

  pub async fn from_stripe_event(&self, e: &stripe::Event) -> Result<Option<Payment>> {
    use stripe::{EventType, EventObject};

    if let (EventType::PaymentIntentSucceeded, EventObject::PaymentIntent(i)) = (&e.event_type, &e.data.object) {
      let customer_id = i.customer.as_ref().map(|c| c.id().to_string() ).ok_or(Error::validation("customer","missing"))?;
      let maybe_org = self.state.org()
        .select()
        .stripe_customer_id_eq(customer_id)
        .optional()
        .await?;

      if let Some(org) = maybe_org {
        let subscription = match org.subscription().await? {
          Some(s) => s,
          _ => return Ok(None),
        };

        let amount = Decimal::new(i.amount_received.ok_or(Error::validation("amount_paid", "missing"))?, 2);

        let maybe_invoice = self.state.invoice()
          .select()
          .amount_eq(&amount)
          .org_id_eq(org.id())
          .payment_id_is_set(false)
          .payment_source_eq(&PaymentSource::Stripe)
          .optional()
          .await?;

        let price = subscription.attrs.price_per_token;

        let tokens = maybe_invoice
          .as_ref()
          .map(|i| i.attrs.tokens )
          .unwrap_or_else(|| (amount / price ).floor() );

        Ok(Some(
          self.insert(InsertPayment{
            org_id: org.attrs.id,
            created_at: Utc::now(),
            amount: amount,
            tokens: tokens,
            fees: Decimal::ZERO,
            payment_source: PaymentSource::Stripe,
            clearing_data: serde_json::to_string(&i)?,
            invoice_id: maybe_invoice.map(|i| i.attrs.id),
          }).save_and_trigger_updates().await?
        ))
      } else {
        Ok(None)
      }
    } else {
      Ok(None)
    }
  }
}

