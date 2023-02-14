use super::*;
use models::{
  PaymentSource,
  Decimal
};

#[derive(Clone, GraphQLInputObject, Serialize, Deserialize)]
#[graphql(description = "InvoiceLinkInput Object")]
#[serde(rename_all = "camelCase")]
pub struct InvoiceLinkInput {
  #[graphql(description = "can be 'Bitcoin' or 'Stripe'")]
  pub payment_method: String,
  #[graphql(description = "amount of token to buy")]
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
#[graphql(description = "InvoiceLink Object")]
pub struct InvoiceLink {
  #[graphql(description = "number identifying an invoice link")]
  pub id: i32,
  #[graphql(description = "minimum amount of token suggested to the user to buy taking into account missing tokens")]
  pub minimum_suggested: i32,
  #[graphql(description = "price per token the user is going to pay")]
  pub price_per_token: i32,
  #[graphql(description = "url of the invoice created with this invoice link if there is any")]
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