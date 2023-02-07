use super::*;


#[derive(GraphQLObject)]
#[graphql(description = "A telegram user")]
pub struct Telegram {
  id: String,
  person_id: i32,
  org_id: i32,
  username: Option<String>,
  first_name: String,
  last_name: Option<String>,
}

#[derive(Clone, GraphQLInputObject, Debug)]
pub struct TelegramFilter {
  ids: Option<Vec<String>>,
  id_like: Option<String>,
  person_id_eq: Option<i32>,
  org_id_eq: Option<i32>,
  username_like: Option<String>,
  first_name_like: Option<String>,
  last_name_like: Option<String>,
}

#[rocket::async_trait]
impl Showable<telegram::TelegramUser, TelegramFilter> for Telegram {
  fn sort_field_to_order_by(field: &str) -> Option<TelegramUserOrderBy> {
    match field {
      "id" => Some(TelegramUserOrderBy::Id),
      "personId" => Some(TelegramUserOrderBy::PersonId),
      "orgId" => Some(TelegramUserOrderBy::OrgId),
      "firstName" => Some(TelegramUserOrderBy::FirstName),
      "username" => Some(TelegramUserOrderBy::Username),
      "lastName" => Some(TelegramUserOrderBy::LastName),
      _ => None,
    }
  }

  fn filter_to_select(f: TelegramFilter) -> SelectTelegramUser {
    SelectTelegramUser{
      id_in: f.ids,
      id_ilike: into_like_search(f.id_like),
      person_id_eq: f.person_id_eq,
      org_id_eq: f.org_id_eq,
      username_like: into_like_search(f.username_like),
      first_name_like: into_like_search(f.first_name_like),
      last_name_like: into_like_search(f.last_name_like),
      ..Default::default()
    }
  }

  async fn db_to_graphql(d: telegram::TelegramUser ) -> MyResult<Self> {
    Ok(Telegram {
      id: d.attrs.id,
      person_id: d.attrs.person_id,
      org_id: d.attrs.org_id,
      username: d.attrs.username,
      first_name: d.attrs.first_name,
      last_name: d.attrs.last_name,
    })
  }
}
