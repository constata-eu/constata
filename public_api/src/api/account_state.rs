use super::*;

#[derive(Debug, GraphQLObject, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
#[graphql(description = "This is your organization's account information and stats.")]
pub struct AccountState {
  #[graphql(description = "Unique ID of your organization assigned by us.")]
  id: i32,
  #[graphql(description = "Tokens your organization needs to buy in order to certify all parked documents created through Issuances, Attestations or any other means.")]
  missing: i32,
  #[graphql(description = "You have this many tokens in your balance.")]
  pub token_balance: i32,
  #[graphql(description = "The special price you'll pay for each token when you buy them.")]
  price_per_token: i32,
  #[graphql(description = 
    "Constata will give you a monthly bonus of this many tokens, as a gift. \
    When you submit an Issuance, Attestation or perform any other token consuming action, \
    we'll credit these tokens as a gift instead of consuming your token balance.")]
  max_monthly_gift: i32,
  #[graphql(description = "This number lowers every time Constata gives you a token from your max_monthly_gift")]
  monthly_gift_remainder: i32,
  #[graphql(description = "You have this many documents parked, that is, they won't be certified, until you take the requested action which may be buying the tokens needed to certify them, or accept changes to our terms and conditions.")]
  parked_count: i32,
  #[graphql(description = "These are all the invoices you created for purchasing tokens and are still waiting for payment.")]
  invoices: Vec<Invoice>,
  #[graphql(description = "When this url is available, you should visit it on your browser to review and accept our terms and conditions. You can send it to whoever is in charge of accepting the terms in your organization.")]
  pending_tyc_url: Option<String>,
  #[graphql(description = "Whenever you are missing tokens, you can visit this url in your browser and it will present you with payment options to buy the tokens you need. No login required, so you can send it to anyone in your organization in charge of payments.")]
  pending_invoice_link_url: Option<String>,
  #[graphql(description = "This should be an address on your own website such as 'https://yoursite.com/web_callbacks_from_constata' where we can notify you about done attestations, issuances, or pending token purchases.")]
  web_callbacks_url: Option<String>,
  #[graphql(description = "A custom logo URL for your organization, configured by us.")]
  logo_url: Option<String>,
  #[graphql(description = "Whether your account has access to the Credentials Verifier feature.")]
  use_verifier: bool,
}

#[derive(Debug, GraphQLObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[graphql(description = "An invoice generated when you chose a payment method and amount of tokens to buy, that has not been paid yet.")]
pub struct Invoice {
  #[graphql(description = "Amount to pay, in EUR")]
  amount: i32,
  #[graphql(description = "Tokens that you'll receive.")]
  tokens: i32,
  #[graphql(description = "Extra data about this invoice, if any.")]
  description: String,
  #[graphql(description = "Visit this url to continue your payment.")]
  url: String,
}

impl AccountState {
  pub fn from_db(d: db::AccountState) -> FieldResult<Self> {
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
      web_callbacks_url: d.org.attrs.web_callbacks_url,
      logo_url: d.org.attrs.logo_url,
      use_verifier: d.org.attrs.use_verifier,
    })
  }
}
