use super::*;
use db::*;

#[derive(GraphQLObject)]
#[graphql(description = "A payment")]
pub struct Payment {
  id: i32,
  org_id: i32,
  created_at: UtcDateTime,
  amount:  i32,
  tokens: i32,
  fees: Option<i32>,
  payment_source: Option<PaymentSource>,
  clearing_data: Option<String>,
  invoice_id: Option<i32>,
}

#[derive(Clone, GraphQLInputObject, Debug)]
pub struct PaymentFilter {
  ids: Option<Vec<i32>>,
  id_eq: Option<i32>,
  org_id_eq: Option<i32>,
  invoice_id_eq: Option<i32>,
  payment_source_eq: Option<String>,
  tokens_eq: Option<i32>,
  tokens_gt: Option<i32>,
  tokens_lt: Option<i32>,
  amount_eq: Option<i32>,
  amount_gt: Option<i32>,
  amount_lt: Option<i32>,
  created_at_eq: Option<UtcDateTime>,
  created_at_lte: Option<UtcDateTime>,
}

#[rocket::async_trait]
impl Showable<db::Payment, PaymentFilter> for Payment {
  fn sort_field_to_order_by(field: &str) -> Option<PaymentOrderBy> {
    match field {
      "id" => Some(PaymentOrderBy::Id),
      "orgId" => Some(PaymentOrderBy::OrgId),
      "invoiceId" => Some(PaymentOrderBy::InvoiceId),
      "amount" => Some(PaymentOrderBy::Amount),
      "tokens" => Some(PaymentOrderBy::Tokens),
      "paymentSource" => Some(PaymentOrderBy::PaymentSource),
      "createdAt" => Some(PaymentOrderBy::CreatedAt),
      _ => None,
    }
  }

  fn filter_to_select(f: PaymentFilter) -> SelectPayment {
    let payment_source_eq = f.payment_source_eq.and_then(|p|
      match p.as_str() {
        "BankBbva" => Some(PaymentSource::BankBbva),
        "Stripe" => Some(PaymentSource::Stripe),
        "BtcPay" => Some(PaymentSource::BtcPay),
        _ => None,
      }
    );

    SelectPayment{
      id_in: f.ids,
      id_eq: f.id_eq,
      org_id_eq: f.org_id_eq,
      invoice_id_eq: f.invoice_id_eq,
      payment_source_eq,
      tokens_eq: into_decimal(f.tokens_eq),
      tokens_gt: into_decimal(f.tokens_gt),
      tokens_lt: into_decimal(f.tokens_lt),
      amount_eq: into_decimal(f.amount_eq),
      amount_gt: into_decimal(f.amount_gt),
      amount_lt: into_decimal(f.amount_lt),
      created_at_eq: f.created_at_eq,
      created_at_lte: f.created_at_lte,
      ..Default::default()
    }
  }

  async fn db_to_graphql(d: db::Payment ) -> ConstataResult<Self> {
    Ok(Payment {
      id: d.attrs.id,
      org_id: d.attrs.org_id,
      created_at: d.attrs.created_at,
      amount:  d.attrs.amount.to_i32().unwrap_or(0),
      tokens: d.attrs.tokens.to_i32().unwrap_or(0),
      fees: Some(d.attrs.fees.to_i32().unwrap_or(0)),
      payment_source: Some(d.attrs.payment_source),
      clearing_data: Some(d.attrs.clearing_data),
      invoice_id: d.attrs.invoice_id,
    })
  }
}
