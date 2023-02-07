use super::*;
use models::{
  PaymentSource,
  Decimal
};

#[derive(Clone, GraphQLInputObject, Serialize, Deserialize)]
#[graphql(description = "An invoice link input")]
#[serde(rename_all = "camelCase")]
pub struct InvoiceLinkInput {
  pub payment_method: String,
  pub tokens: i32,
}

impl InvoiceLinkInput {
  pub async fn process(self, context: &Context) -> FieldResult<InvoiceLink> {
    if let AuthMethod::Token { ref token } = context.current_person.method {
      let payment_source = match self.payment_method.as_str() {
        "Bitcoin" => PaymentSource::BtcPay,
        _ => PaymentSource::Stripe
      };
      
      context.site.invoice_link().select()
        .access_token_id_eq(token.id())
        .invoice_id_is_set(false)
        .one().await?
        .make_invoice(payment_source, Decimal::new(self.tokens.into(), 0)).await?;
      return InvoiceLink::invoice_link(context).await
    } else {
      Err(field_error("access", "nothing to create"))
    }
  }
}

#[derive(GraphQLObject)]
#[graphql(description = "An invoice link")]
pub struct InvoiceLink {
  pub id: i32,
  pub minimum_suggested: i32,
  pub price_per_token: i32,
  pub invoice_url: Option<String>,
}

impl InvoiceLink {
  pub async fn invoice_link(context: &Context) -> FieldResult<InvoiceLink> {
    if let AuthMethod::Token { ref token } = context.current_person.method {
      let invoice_link = context.site.invoice_link().select().access_token_id_eq(token.id()).one().await?;
      let org = context.org().await?;
      let invoice_url = match invoice_link.invoice().await? {
        Some(i) => Some(i.attrs.url),
        None => None,
      };
      Ok(InvoiceLink{
        id: invoice_link.attrs.id,
        minimum_suggested: org.account_state().await?.missing.to_i32().unwrap_or(0),
        price_per_token: (org.subscription_or_err().await?.price_per_token() * Decimal::new(100, 0)).to_i32().unwrap_or(0),
        invoice_url,
      })
    } else {
      Err(field_error("access", "nothing to verify"))
    }
  }
}