use super::*;

model!{
  state: Site,
  table: invoices,
  struct Invoice {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    org_id: i32,
    created_at: UtcDateTime,
    #[sqlx_model_hints(payment_source)]
    payment_source: PaymentSource,
    #[sqlx_model_hints(varchar)]
    external_id: String,
    #[sqlx_model_hints(decimal)]
    amount: Decimal,
    #[sqlx_model_hints(decimal)]
    tokens: Decimal,
    description: String,
    url: String,
    #[sqlx_model_hints(boolean)]
    paid: bool,
    #[sqlx_model_hints(boolean)]
    expired: bool,
    #[sqlx_model_hints(int4)]
    payment_id: Option<i32>,
    notified_on: Option<UtcDateTime>,
  },
  belongs_to {
    Org(org_id),
    Payment(payment_id),
  }
}

impl InvoiceHub {
  pub async fn once(self, org: &Org, payment_source: PaymentSource, tokens: Decimal) -> ConstataResult<Invoice> {
    let token_price = org.subscription_or_err().await?.attrs.price_per_token;
    let (url, external_id) = match payment_source {
      PaymentSource::Stripe => self.request_on_stripe(org.clone(), &token_price, &tokens).await?,
      PaymentSource::BtcPay => self.request_on_btc_pay(&token_price, &tokens).await?,
      PaymentSource::BankBbva => ("".to_string(), "".to_string()),
    };

    let invoice = self.insert(InsertInvoice{
      org_id: *org.id(),
      amount: tokens * token_price,
      tokens,
      payment_source,
      url,
      external_id,
      description: format!("{} Tokens", tokens),
      created_at: Utc::now(),
      paid: false,
      expired: false,
      payment_id: None,
      notified_on: None
    }).save().await?;

    Ok(invoice)
  }

  async fn request_on_stripe(&self, org: Org, token_price: &Decimal, tokens: &Decimal) -> ConstataResult<(String, String)> {
    use serde_json::json;
    use stripe::{CheckoutSession, CustomerId};
    use num_traits::cast::ToPrimitive;

    let client = &self.state.stripe_client;
    let customer_id: CustomerId = org.get_or_create_stripe_customer_id().await?;
    let settings = &self.state.settings;

    let stripe_session : CheckoutSession = client.post_form("/checkout/sessions", json![{
      "success_url": settings.payment_success_redirect.clone(),
      "cancel_url": settings.payment_error_redirect.clone(),
      "customer": customer_id,
      "payment_method_types": ["card"],
      "mode": "payment",
      "line_items": [
        {
          "quantity": tokens,
          "price_data": {
            "product": settings.stripe.token_product_code,
            "currency": "EUR",
            "unit_amount": (token_price * Decimal::new(100, 0)).to_i64(), // This amount is expressed in cents.
          }
        }
      ]
    }]).await?;

    Ok((stripe_session.url, stripe_session.id.to_string()))
  }

  async fn request_on_btc_pay(&self, token_price: &Decimal, tokens: &Decimal) -> ConstataResult<(String, String)> {
    let total = tokens * token_price;
    let settings = &self.state.settings;

    let invoice: btcpay::Invoice = ureq::post(&format!(
        "{}/api/v1/stores/{}/invoices",
        settings.btcpay.base_url,
        settings.btcpay.store_id,
      ))
      .set("Authorization", &format!("token {}", settings.btcpay.api_key))
      .send_json(serde_json::to_value(btcpay::InvoiceForm{
        amount: total,
        currency: btcpay::Currency::Eur,
        checkout: btcpay::InvoiceFormCheckout{
          redirectURL: settings.payment_success_redirect.clone(),
          expirationMinutes: 1440,
        }
      })?)?
      .into_json()?;

    Ok((invoice.checkout_link, invoice.id))
  }

  pub async fn expire_all_old_invoices(self) -> sqlx::Result<Self> {
    for invoice in self.select().paid_eq(false).expired_eq(false).all().await? {
      if *invoice.payment_source() != PaymentSource::BankBbva &&
         *invoice.created_at() < Utc::now() - Duration::hours(25) {
        invoice.update().expired(true).save().await?;
      }
    }

    Ok(self)
  }
}

impl Invoice {
  pub async fn make_payment(&mut self, clearing_data: Option<&str>) -> ConstataResult<Payment> {
    let payment = self.state.payment()
      .insert(InsertPayment{
        org_id: self.attrs.org_id,
        created_at: Utc::now(),
        amount: self.attrs.amount,
        tokens: self.attrs.tokens,
        fees: Decimal::ZERO,
        payment_source: self.attrs.payment_source,
        clearing_data: clearing_data.unwrap_or("").to_string(),
        invoice_id: Some(self.attrs.id),
      }).save_and_trigger_updates().await?;
    self.reload().await?;
    Ok(payment)
  }

  pub async fn expire(self) -> sqlx::Result<Self> {
    self.update().expired(true).save().await
  }
}

describe! {
  dbtest!{ can_turn_into_payment (site, c)
    c.alice().await;
    let org_id = c.enterprise().await.org().await.attrs.id;

    let mut invoice = site.invoice().insert(InsertInvoice{
      org_id: org_id,
      created_at: Utc::now(),
      payment_source: PaymentSource::BtcPay,
      external_id: "something".to_string(),
      amount: Decimal::new(10,0),
      tokens: Decimal::new(5,0),
      description: "Some charges".to_string(),
      url: "https://example.com".to_string(),
      paid: false,
      expired: false,
      payment_id: None,
      notified_on: None,
    }).save().await?;

    let payment = invoice.make_payment(Some("Payment done")).await?;

    assert_that!(&invoice.attrs, structure![ InvoiceAttrs {
      paid: eq(true),
      expired: eq(false),
      payment_id: maybe_some(eq(payment.attrs.id)),
    }]);

    assert_that!(&payment.attrs, structure![ PaymentAttrs {
      id: eq(1),
      org_id: eq(*org_id),
      amount: eq(Decimal::new(10,0)),
      tokens: eq(Decimal::new(5,0)),
      fees: eq(Decimal::ZERO),
      payment_source: eq(PaymentSource::BtcPay),
      clearing_data: eq("Payment done".to_string()),
      invoice_id: maybe_some(eq(invoice.attrs.id)),
    }]);

    assert_eq!(payment, site.payment().select().org_id_eq(&org_id).one().await?);
  }

  dbtest!{ expires_unpaid_old_invoices (site, c)
    let alice = c.alice().await;
    site.invoice().expire_all_old_invoices().await?;
    assert_counter_invoices_not_expired_not_paid(&site, 0, None).await?;

    let mut invoices = vec![];
    invoices.push(alice.make_invoice_with_payment_source(PaymentSource::BankBbva).await);
    invoices.push(alice.make_invoice_with_payment_source(PaymentSource::BtcPay).await);
    invoices.push(alice.make_invoice_with_payment_source(PaymentSource::BtcPay).await);
    invoices.push(alice.make_invoice_with_payment_source(PaymentSource::BtcPay).await);
    invoices.push(alice.make_invoice_with_payment_source(PaymentSource::BtcPay).await);
    let last_invoice = alice.make_invoice_with_payment_source(PaymentSource::BtcPay).await;
    let stripe_invoice = alice.make_invoice().await;
    invoices[1].make_payment(Some("Payment")).await?;

    for invoice in invoices {
      invoice.update().created_at(Utc::now() - Duration::hours(26)).save().await?;
    }

    site.invoice().expire_all_old_invoices().await?;
    assert_counter_invoices_when_expired_is(&site, true, 3, Some(PaymentSource::BtcPay)).await?;
    assert_counter_invoices_when_expired_is(&site, false, 2, Some(PaymentSource::BtcPay)).await?;
    assert_counter_invoices_not_expired_not_paid(&site, 1, Some(PaymentSource::BtcPay)).await?;
 

    last_invoice.update().created_at(Utc::now() - Duration::hours(26)).save().await?;
    site.invoice().expire_all_old_invoices().await?;
    assert_counter_invoices_not_expired_not_paid(&site, 0, Some(PaymentSource::BtcPay)).await?;
    assert_counter_invoices_not_expired_not_paid(&site, 1, Some(PaymentSource::BankBbva)).await?;
    assert_counter_invoices_not_expired_not_paid(&site, 1, Some(PaymentSource::Stripe)).await?;
    assert_counter_invoices_not_expired_not_paid(&site, 2, None).await?;

    stripe_invoice.update().created_at(Utc::now() - Duration::hours(26)).save().await?;
    site.invoice().expire_all_old_invoices().await?;
    assert_counter_invoices_not_expired_not_paid(&site, 1, None).await?;
  }

  async fn assert_counter_invoices_not_expired_not_paid(site: &Site, counter: i64, payment_source: Option<PaymentSource>) -> ConstataResult<()> {
    let invoices_counter = match payment_source {
      Some(x) => site.invoice().select().expired_eq(false).paid_eq(false).payment_source_eq(x).count().await?,
      None => site.invoice().select().expired_eq(false).paid_eq(false).count().await?,
    };
    assert_eq!(invoices_counter, counter);
    Ok(())
  }
  async fn assert_counter_invoices_when_expired_is(site: &Site, expired: bool, counter: i64, payment_source: Option<PaymentSource>)
    -> ConstataResult<()> {
    let invoices_counter = match payment_source {
      Some(x) => site.invoice().select().expired_eq(expired).payment_source_eq(x).count().await?,
      None => site.invoice().select().expired_eq(expired).count().await?,
    };
    assert_eq!(invoices_counter, counter);
    Ok(())
  }
}
