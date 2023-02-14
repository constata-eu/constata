use super::*;
use crate::Decimal;

#[derive(GraphQLObject)]
#[graphql(description = "AccountState Object")]
pub struct AccountState {
  #[graphql(description = "number identifying the organization")]
  id: i32,
  #[graphql(description = "tokens left to certify user's pending documents")]
  missing: i32,
  #[graphql(description = "user's token balance")]
  token_balance: i32,
  #[graphql(description = "price per token the user is going to pay")]
  price_per_token: i32,
  #[graphql(description = "amount of token per mounth gifted to the user")]
  max_monthly_gift: i32,
  #[graphql(description = "amount of tokens gifted this month that I have available")]
  monthly_gift_remainder: i32,
  #[graphql(description = "amount of document pending by lack of tokens")]
  parked_count: i32,
  #[graphql(description = "all user's pending invoices")]
  invoices: Vec<Invoice>,
  #[graphql(description = "It's going to return the url to accept terms & conditions if user didn't accept them yet")]
  pending_tyc_url: Option<String>,
  #[graphql(description = "It's going to return the url to an invoice link if user didn't create an invoice with it")]
  pending_invoice_link_url: Option<String>,
}

#[derive(GraphQLObject)]
#[graphql(description = "Invoice Object")]
pub struct Invoice {
  #[graphql(description = "amount of money to pay for the tokens")]
  amount: i32,
  #[graphql(description = "amount of token to buy")]
  tokens: i32,
  #[graphql(description = "description of the invoice")]
  description: String,
  #[graphql(description = "url where to buy the tokens")]
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
