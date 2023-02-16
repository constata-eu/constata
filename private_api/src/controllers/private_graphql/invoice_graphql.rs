use super::*;


#[derive(GraphQLObject, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[graphql(description = "An invoice")]
pub struct Invoice {
  id: i32,
  org_id: i32,
  created_at: UtcDateTime,
  amount:  i32,
  tokens: i32,
  payment_source: Option<PaymentSource>,
  description: Option<String>,
  external_id: Option<String>,
  url: Option<String>,
  notified_on: Option<UtcDateTime>,
  paid: Option<bool>,
  payment_id: Option<i32>,
  expired: Option<bool>,
}

#[derive(Clone, GraphQLInputObject, Debug)]
pub struct InvoiceFilter {
  ids: Option<Vec<i32>>,
  id_eq: Option<i32>,
  org_id_eq: Option<i32>,
  payment_id_eq: Option<i32>,
  payment_source_eq: Option<String>,
  tokens_eq: Option<i32>,
  tokens_gt: Option<i32>,
  tokens_lt: Option<i32>,
  amount_eq: Option<i32>,
  amount_gt: Option<f64>,
  amount_lt: Option<f64>,
  paid_eq: Option<bool>,
  expired_eq: Option<bool>,
}

#[rocket::async_trait]
impl Showable<invoice::Invoice, InvoiceFilter> for Invoice {
  fn sort_field_to_order_by(field: &str) -> Option<InvoiceOrderBy> {
    match field {
      "id" => Some(InvoiceOrderBy::Id),
      "orgId" => Some(InvoiceOrderBy::OrgId),
      "paymentId" => Some(InvoiceOrderBy::PaymentId),
      "paymentSource" => Some(InvoiceOrderBy::PaymentSource),
      "amount" => Some(InvoiceOrderBy::Amount),
      "tokens" => Some(InvoiceOrderBy::Tokens),
      "paid" => Some(InvoiceOrderBy::Paid),
      "expired" => Some(InvoiceOrderBy::Expired),
      "createdAt" => Some(InvoiceOrderBy::CreatedAt),
      _ => None,
    }
  }

  fn filter_to_select(f: InvoiceFilter) -> SelectInvoice {
    let payment_source_eq = f.payment_source_eq.and_then(|s|{
      match s.as_str() {
        "BankBbva" => Some(PaymentSource::BankBbva),
        "Stripe" => Some(PaymentSource::Stripe),
        "BtcPay" => Some(PaymentSource::BtcPay),
        _ => None,
      }
    });

    SelectInvoice{
      id_in: f.ids,
      id_eq: f.id_eq,
      payment_id_eq: f.payment_id_eq,
      org_id_eq: f.org_id_eq,
      paid_eq: f.paid_eq,
      expired_eq: f.expired_eq,
      payment_source_eq,
      tokens_eq: into_decimal(f.tokens_eq),
      tokens_gt: into_decimal(f.tokens_gt),
      tokens_lt: into_decimal(f.tokens_lt),
      amount_eq: into_decimal(f.amount_eq),
      amount_gt: into_decimal(f.amount_gt),
      amount_lt: into_decimal(f.amount_lt),
      ..Default::default()
    }
  }

  async fn db_to_graphql(d: invoice::Invoice ) -> MyResult<Self> {
    use rust_decimal::prelude::ToPrimitive;
    Ok(Invoice {
      id: d.attrs.id,
      org_id: d.attrs.org_id,
      created_at: d.attrs.created_at,
      amount: d.attrs.amount.to_f32().unwrap_or(0),
      tokens: d.attrs.tokens.to_f32().unwrap_or(0),
      payment_source: Some(d.attrs.payment_source),
      description: Some(d.attrs.description),
      external_id: Some(d.attrs.external_id),
      url: Some(d.attrs.url),
      notified_on: d.attrs.notified_on,
      paid: Some(d.attrs.paid),
      payment_id: d.attrs.payment_id,
      expired: Some(d.attrs.expired),
    })
  }
}
