use super::*;
use rust_decimal::prelude::ToPrimitive;
use juniper::{
  FieldResult,
  FieldError,
  graphql_value,
  GraphQLObject,
};

const DEFAULT_PER_PAGE: i32 = 20;
const DEFAULT_PAGE: i32 = 0;

#[rocket::async_trait]
pub trait Showable<Model: SqlxModel<State=Site>, Filter: Send>: Sized {
  fn sort_field_to_order_by(field: &str) -> Option<<Model as SqlxModel>::ModelOrderBy>;
  fn filter_to_select(org_id: i32, f: Option<Filter>) -> <Model as SqlxModel>::SelectModel;
  fn select_by_id(org_id: i32, id: <Model as SqlxModel>::Id) -> <Model as SqlxModel>::SelectModel;
  async fn db_to_graphql(d: Model) -> ConstataResult<Self>;

  async fn resource(context: &Context, id: <Model as SqlxModel>::Id) -> FieldResult<Self> 
    where <Model as SqlxModel>::Id: 'async_trait
  {
    let resource = <<Model as SqlxModel>::ModelHub>::from_state(context.site.clone())
      .select()
      .use_struct(Self::select_by_id(context.org_id(), id))
      .one()
      .await?;
    Ok(Self::db_to_graphql(resource).await?)
  }

  async fn collection(
    context: &Context,
    page: Option<i32>,
    per_page: Option<i32>,
    sort_field: Option<String>,
    sort_order: Option<String>,
    filter: Option<Filter>
  ) -> FieldResult<Vec<Self>>
    where Filter: 'async_trait
  {
    let limit = per_page.unwrap_or(DEFAULT_PER_PAGE);
    if limit >= 500 {
      return Err(FieldError::new(
        "Invalid pagination",
        graphql_value!({ "internal_error": "Invalid pagination" })
      ));
    }
    let offset = page.unwrap_or(DEFAULT_PAGE) * limit;

    let maybe_order_by = match sort_field {
      None => None,
      Some(ref field) => {
        if let Some(order_by) = Self::sort_field_to_order_by(field) {
          Some(order_by)
        } else {
          return Err(FieldError::new("Invalid sort_field", graphql_value!({ "invalid_sort": format!("{:?}", &sort_field) })))
        }
      }
    }; 

    let selected = <Model as SqlxModel>::SelectModelHub::from_state(context.site.clone())
      .use_struct( Self::filter_to_select(context.org_id(), filter) )
      .maybe_order_by(maybe_order_by)
      .limit(limit.into())
      .offset(offset.into())
      .desc(sort_order == Some("DESC".to_string()))
      .all().await?;

    let mut all = vec![];
    for p in selected.into_iter() {
      all.push(Self::db_to_graphql(p).await?);
    }
    Ok(all)
  }

  async fn count( context: &Context, filter: Option<Filter>) -> FieldResult<ListMetadata>
    where Filter: 'async_trait
  {
    let count = <Model as SqlxModel>::SelectModelHub::from_state(context.site.clone())
      .use_struct( Self::filter_to_select(context.org_id(), filter) )
      .count().await?
      .to_i32()
      .ok_or(FieldError::new("too_many_records", graphql_value!({})))?;

    Ok(ListMetadata{count})
  }
}

#[derive(Debug, GraphQLObject, serde::Serialize, serde::Deserialize)]
pub struct ListMetadata {
  pub count: i32
}
