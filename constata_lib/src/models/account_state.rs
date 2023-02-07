use super::{*, invoice::*};
use crate::Result;


#[derive(Debug, Serialize)]
pub struct AccountState {
  #[serde(skip_serializing)]
  pub state: Site,
  #[serde(skip_serializing)]
  pub org: Org,
  pub org_id: i32,
  pub token_balance: Decimal,
  pub missing: Decimal,
  pub price_per_token: Decimal,
  pub max_monthly_gift: Decimal,
  pub monthly_gift_remainder: Decimal,
  pub invoices: Vec<Invoice>,
  pub funded_count: i64,
  pub funded_tokens: Decimal,
  pub parked_count: i64,
  pub parked_tokens: Decimal,
  pub funded_documents_count: i64,
  pub total_document_count: i64,
  pub total_document_tokens: Decimal,
  pub pending_tyc_url: Option<String>,
  pub pending_invoice_link_url: Option<String>,
  pub parked_documents_urls: Vec<(String, String)>,
}

impl AccountState {
  pub async fn find_for(state: Site, org_id: i32) -> Result<AccountState> {
    let total_document_count: i64 = state.db.fetch_one_scalar(sqlx::query_scalar!(
      r#"SELECT COUNT(*)::bigint as "count!" FROM documents WHERE org_id = $1"#,
      org_id,
    )).await?;

    let bought: Decimal = state.db.fetch_one_scalar(sqlx::query_scalar!(
      r#"SELECT COALESCE(SUM(tokens), 0)::decimal as "bought!" FROM payments WHERE org_id = $1"#,
      org_id,
    )).await?;

    let gifts: Decimal = state.db.fetch_one_scalar(sqlx::query_scalar!(
      r#"SELECT COALESCE(SUM(tokens), 0)::decimal as "gifts!" FROM gifts WHERE org_id = $1"#,
      org_id,
    )).await?;

    let spent: Decimal = state.db.fetch_one_scalar(sqlx::query_scalar!(
      r#"SELECT COALESCE(SUM(cost), 0)::decimal as "spent!" FROM documents WHERE funded AND org_id = $1"#,
      org_id,
    )).await?;

    let unfunded: Decimal = state.db.fetch_one_scalar(sqlx::query_scalar!(
      r#"SELECT COALESCE(SUM(cost), 0)::decimal as "missing_tokens!" FROM documents WHERE NOT funded AND org_id = $1"#,
      org_id,
    )).await?;

    let funded_count: i64 = state.db.fetch_one_scalar(sqlx::query_scalar!(
      r#"SELECT COUNT(*)::bigint as "count!" FROM documents WHERE funded AND org_id = $1"#,
      org_id,
    )).await?;

    let parked_count: i64 = state.db.fetch_one_scalar(sqlx::query_scalar!(
      r#"SELECT COUNT(*)::bigint as "count!" FROM documents WHERE NOT funded AND org_id = $1"#,
      org_id,
    )).await?;

    let funded_documents_count: i64 = state.db.fetch_one_scalar(sqlx::query_scalar!(
      r#"SELECT COUNT(*)::bigint as "count!" FROM documents WHERE funded AND org_id = $1"#,
      org_id,
    )).await?;

    let invoices = state.invoice()
      .select()
      .payment_id_is_set(false)
      .expired_eq(&false)
      .org_id_eq(&org_id)
      .all()
      .await?;

    let token_balance = bought + gifts - spent;
    let missing = (unfunded - token_balance).max(Decimal::ZERO);

    let org = state.org().find(&org_id).await?;
    let subscription = org.subscription_or_err().await?;
    let price_per_token = subscription.attrs.price_per_token;
    let max_monthly_gift = subscription.attrs.max_monthly_gift;
    let monthly_gift_remainder = subscription.monthly_gift_remainder().await?;
    let tyc = org.get_or_create_terms_acceptance().await?;
    let pending_tyc_url = if tyc.is_needed() { Some(tyc.full_url()) } else { None };
    let parked_documents_urls = org.get_parked_documents_urls().await?;
    let pending_invoice_link_url = if missing.is_zero() {
      None
    } else {
      Some(org.get_or_create_invoice_link().await?.buy_tokens_link().await)
    };

    Ok(AccountState{
      state,
      org,
      org_id,
      token_balance,
      invoices,
      missing,
      price_per_token,
      max_monthly_gift,
      monthly_gift_remainder,
      parked_count,
      parked_tokens: unfunded,
      funded_count,
      funded_tokens: spent,
      funded_documents_count,
      total_document_count,
      total_document_tokens: unfunded + spent,
      pending_tyc_url,
      pending_invoice_link_url,
      parked_documents_urls,
    })
  }
}

impl AccountState {
  pub async fn fund_all_documents(self) -> Result<Vec<String>> {
    let unfunded = self.state.db.fetch_all(sqlx::query!(
      r#"SELECT id, cost::decimal as "cost!" FROM documents WHERE org_id = $1 AND NOT funded ORDER BY gift_id NULLS LAST, created_at"#,
      self.org_id
    )).await?;

    let mut tokens = self.token_balance;
    let mut funded_ids = vec![];

    let org = self.state.org().find(&self.org_id).await?;
      
    if org.get_or_create_terms_acceptance().await?.is_needed() {
      return Ok(funded_ids)
    }

    for doc in unfunded {
      if tokens < doc.cost {
        break;
      }

      self.state.db.execute(sqlx::query!(
        "UPDATE documents SET funded = true, funded_at = now(), bulletin_id = current_draft()
          WHERE id = $1
        ",
        doc.id
      )).await?;

      self.state.db.execute(sqlx::query!(
        "UPDATE document_part_signatures as dps SET bulletin_id = current_draft()
        FROM document_parts as dp WHERE dp.id = dps.document_part_id AND dp.document_id = $1",
        doc.id
      )).await?;

      funded_ids.push(doc.id);

      tokens = tokens - doc.cost;
    }

    Ok(funded_ids)
  }
}

describe! {
  use rust_decimal_macros::dec;

  dbtest!{ shows_funding_history_and_funds_documents (site, c)
    let user = c.alice().await;
    let org = user.org().await;

    macro_rules! assert_account_state (
      ($balance:expr, $miss:expr, $invoices:expr, $parked:expr) => (
        {
          let state = org.account_state().await?;
          assert_that!(&state, structure![ AccountState{
            token_balance: eq(Decimal::new($balance,0)),
            missing: eq(Decimal::new($miss,0)),
            parked_count: eq($parked),
          }]);
          assert_eq!(&state.invoices, &$invoices);
        }
      )
    );

    assert_account_state!(0, 0, vec![], 0);

    // A new document is created for 1 token, which is gifted to the user.
    let first = user.signed_document(b"hello world").await.in_accepted()?;
    assert_that!(&first.as_inner().cost(), eq(dec!(1)));
    assert_account_state!(0, 0, vec![], 0);
    
    // The second document needs more tokens that what can be given as a gift.
    // So it remains parked.
    let second = user.signed_document(&vec![0; 1024 * 1024 * 10]).await.in_parked()?;
    assert_that!(&second.as_inner().cost(), eq(dec!(10)));

    assert_account_state!(9, 1, vec![], 1);

    // Then a third document adds to the unpaid list.
    let third = user.signed_document(&vec![0; 1024 * 1024 * 11]).await.in_parked()?;
    assert_that!(&third.as_inner().cost(), eq(dec!(11)));
    assert_account_state!(9, 12, vec![], 2);

    // A new invoice is created for an amount larger than the missing tokens.
    let mut invoice = site.invoice().once(&org, PaymentSource::BankBbva, Decimal::new(15,0)).await?;
    assert_account_state!(9, 12, vec![invoice.clone()], 2);

    // And before the invoice is paid, a fourth document is received.
    let fourth = user.signed_document(&vec![0; 1024 * 1024 * 5]).await.in_parked()?;
    assert_that!(&fourth.as_inner().cost(), eq(dec!(5)));
    assert_account_state!(9, 17, vec![invoice.clone()], 3);
    assert_eq!(3, org.account_state().await?.parked_documents_urls.len());

    // The payment is made, and all documents but this last one are accepted.
    // The invoice no longer shows as pending.
    invoice.make_payment(Some("Saw it on the bank website")).await?;
    assert!(site.document().find(second.as_inner().id()).await?.is_accepted());
    assert!(site.document().find(third.as_inner().id()).await?.is_accepted());
    assert!(site.document().find(fourth.as_inner().id()).await?.is_parked());
    assert_account_state!(3, 2, vec![], 1);

    // A new document that is less than the available tokens is still not processed
    // to maintain the "first in, firstout" policy.
    let fifth = user.signed_document(&vec![1; 1024]).await.in_parked()?;
    assert_that!(&fifth.as_inner().cost(), eq(dec!(1)));
    assert_account_state!(3, 3, vec![], 2);

    // Adding 99 tokens to this org will get the 4th and 5th docs accepted.
    user.fund().await;
    assert!(site.document().find(fourth.as_inner().id()).await?.is_accepted());
    assert!(site.document().find(fifth.as_inner().id()).await?.is_accepted());
    assert_account_state!(96, 0, vec![], 0);

    // And further documents are accepted immediately as there are enough funds.
    assert!(user.signed_document(b"hi").await.is_accepted());
    assert_account_state!(95, 0, vec![], 0);

    // An invoice can be created even if there are tokens.
    let second_invoice = site.invoice().once(&org, PaymentSource::BankBbva, Decimal::new(15,0)).await?;
    assert_account_state!(95, 0, vec![second_invoice.clone()], 0);

    // And when the invoice is cancelled it doesn't show up anymore.
    second_invoice.expire().await?;
    assert_account_state!(95, 0, vec![], 0);
  }
}
