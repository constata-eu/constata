use super::*;
use crate::Decimal;

#[derive(GraphQLObject)]
#[graphql(description = "The customer account state")]
pub struct AccountState {
  id: i32,
  missing: i32,
  token_balance: i32,
  price_per_token: i32,
  max_monthly_gift: i32,
  monthly_gift_remainder: i32,
  parked_count: i32,
  invoices: Vec<Invoice>,
  pending_tyc_url: Option<String>,
  pending_invoice_link_url: Option<String>,
}

#[derive(GraphQLObject)]
#[graphql(description = "An invoice ready to be paid")]
pub struct Invoice {
  amount: i32,
  tokens: i32,
  description: String,
  url: String,
}


impl AccountState {
  pub fn from_db(d: account_state::AccountState) -> FieldResult<Self> {
    let invoices = d.invoices.into_iter().map(|i|{
      let tokens = i.attrs.tokens.to_i32().ok_or(FieldError::new("too_many_tokens", graphql_value!({})))?;
      let amount = i.attrs.amount.to_i32().ok_or(FieldError::new("amount_too_large", graphql_value!({})))?;
      Ok(Invoice{ amount, tokens, description: i.attrs.description, url: i.attrs.url })
    }).collect::<FieldResult<Vec<Invoice>>>()?;
    let price_per_token = d.price_per_token * Decimal::new(100, 0);
    let max_monthly_gift = d.max_monthly_gift;
    let monthly_gift_remainder = d.monthly_gift_remainder;
    let parked_count = d.parked_count as i32;

    Ok(Self {
      id: d.org_id,
      missing: d.missing.to_i32().unwrap_or(0),
      token_balance: d.token_balance.to_i32().unwrap_or(0),
      price_per_token: price_per_token.to_i32().unwrap_or(0),
      max_monthly_gift: max_monthly_gift.to_i32().unwrap_or(0),
      monthly_gift_remainder: monthly_gift_remainder.to_i32().unwrap_or(0),
      parked_count,
      invoices,
      pending_tyc_url: d.pending_tyc_url,
      pending_invoice_link_url: d.pending_invoice_link_url,
    })
  }
}
