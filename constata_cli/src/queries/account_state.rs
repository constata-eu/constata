use super::*;
use gql_types::account_state_graphql::AccountState as Model;

#[derive(serde::Serialize, clap::Args)]
pub struct AccountState { }

impl AccountState {
  pub fn run(&self, client: &Client) -> ClientResult<Model> {
    client.simple(self, "AccountState", "query{
      AccountState(id: 1) {
        id
        missing
        tokenBalance
        pricePerToken
        maxMonthlyGift
        monthlyGiftRemainder
        parkedCount
        invoices {
          amount
          tokens
          description
          url
          __typename
        }
        pendingTycUrl
        pendingInvoiceLinkUrl
        webCallbacksUrl
        __typename
      }
    }")
  }
}
