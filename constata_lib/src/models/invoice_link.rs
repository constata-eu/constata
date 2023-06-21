use super::*;

model!{
  state: Site,
  table: invoice_links,
  struct InvoiceLink {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    access_token_id: i32,
    #[sqlx_model_hints(int4)]
    org_id: i32,
    #[sqlx_model_hints(int4, default)]
    invoice_id: Option<i32>,
  },
  belongs_to {
    Org(org_id),
    AccessToken(access_token_id),
    Invoice(invoice_id),
  },
}

impl InvoiceLink {
  pub async fn make_invoice(self, payment_source: PaymentSource, tokens: Decimal) -> ConstataResult<Invoice> {
    let invoice = self.state.invoice().once(&self.org().await?, payment_source, tokens).await?;
    self.access_token().await?.expire().await?;
    self.update().invoice_id(Some(invoice.attrs.id)).save().await?;
    Ok(invoice)
  }

  pub async fn token(&self) -> String {
    self.access_token().await.expect("to have an access_token").attrs.token
  }

  pub async fn full_url(&self) -> String {
    format!("{}/#/invoice/{}", &self.state.settings.url, self.token().await)
  }

  pub async fn buy_tokens_link(&self) -> String {
    format!(
      "{}/#/invoice/{}",
      &self.state.settings.url,
      self.token().await
    )
  }
}

impl InsertInvoiceLink {
  pub async fn from_org(org: &Org) -> ConstataResult<Self> {
    let person = org.admin().await?;
    let access_token = org.state.access_token().create(&person, AccessTokenKind::InvoiceLink, Some(30)).await?;

    Ok(Self{ org_id: *org.id(), access_token_id: *access_token.id() })
  }
}

describe! {
  dbtest!{ transform_invoice_link_into_invoice (_site, c)
    let alice = c.alice().await;
    let invoice_link = alice.make_invoice_link().await;
    let invoice = invoice_link.clone().make_invoice(PaymentSource::BankBbva, Decimal::new(10, 0)).await?;
    let updated_invoice_link = invoice_link.reloaded().await?;
    assert_eq!(&updated_invoice_link.invoice_id().unwrap(), invoice.id());
    assert_that!(updated_invoice_link.access_token().await?.expired());
  }
}
