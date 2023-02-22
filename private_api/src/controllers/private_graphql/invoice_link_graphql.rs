use super::*;

#[derive(GraphQLObject)]
#[graphql(description = "An invoice link")]
pub struct InvoiceLink {
  id: i32,
  url: String,
  missing_tokens: i32,
  org_id: i32,
  invoice_id: Option<i32>,
  parked_count: i32,
  parked_tokens: i32,
  first_parked_date: Option<UtcDateTime>,
  last_parked_date: Option<UtcDateTime>,
}

#[derive(Clone, GraphQLInputObject, Debug)]
pub struct InvoiceLinkFilter {
  ids: Option<Vec<i32>>,
  id_eq: Option<i32>,
  org_id_eq: Option<i32>,
  invoice_id_eq: Option<i32>,
  invoice_id_is_set: Option<bool>,
}

#[rocket::async_trait]
impl Showable<invoice_link::InvoiceLink, InvoiceLinkFilter> for InvoiceLink {
  fn sort_field_to_order_by(field: &str) -> Option<InvoiceLinkOrderBy> {
    match field {
      "id" => Some(InvoiceLinkOrderBy::Id),
      "orgId" => Some(InvoiceLinkOrderBy::OrgId),
      "invoiceId" => Some(InvoiceLinkOrderBy::InvoiceId),
      _ => None,
    }
  }

  fn filter_to_select(f: InvoiceLinkFilter) -> SelectInvoiceLink {
    SelectInvoiceLink{
      id_in: f.ids,
      id_eq: f.id_eq,
      invoice_id_eq: f.invoice_id_eq,
      org_id_eq: f.org_id_eq,
      invoice_id_is_set: f.invoice_id_is_set,
      ..Default::default()
    }
  }

  async fn db_to_graphql(d: invoice_link::InvoiceLink ) -> MyResult<Self> {
    let org = d.org().await?;
    let account_state = org.account_state().await?;
    let (first_parked_date, last_parked_date) = org.date_first_and_last_document_parked().await;
    let missing_tokens = account_state.missing.to_i32().unwrap_or(0);
    Ok(InvoiceLink {
      id: d.attrs.id,
      url: d.buy_tokens_link().await,
      org_id: d.attrs.org_id,
      invoice_id: d.attrs.invoice_id,
      missing_tokens,
      parked_count: account_state.parked_count.to_i32().unwrap_or(0),
      parked_tokens: account_state.parked_tokens.to_i32().unwrap_or(0),
      first_parked_date,
      last_parked_date,
    })
  }
}

impl InvoiceLink {
  pub async fn create_invoice_link(context: &Context, org_id: i32) -> FieldResult<InvoiceLink> {
    let org = context.site.org().find(&org_id).await?;
    let db_invoice_link = context.site.invoice_link()
      .insert(InsertInvoiceLink::from_org(&org).await?)
      .save().await?;

    Ok(InvoiceLink::db_to_graphql(db_invoice_link).await?)
  }
}
