use super::*;


#[derive(GraphQLObject, Deserialize)]
#[graphql(description = "Admin Users")]
pub struct AdminUser {
  id: i32,
  username: String,
  hashed_password: String,
  otp_seed: String,
  role: AdminRole,
  created_at: UtcDateTime,
}

#[derive(Clone, GraphQLInputObject, Debug)]
pub struct AdminUserFilter {
  ids: Option<Vec<i32>>,
  username_like: Option<String>,
  role_eq: Option<String>,
}

#[rocket::async_trait]
impl Showable<admin_user::AdminUser, AdminUserFilter> for AdminUser {
  fn sort_field_to_order_by(field: &str) -> Option<AdminUserOrderBy> {
    match field {
      "id" => Some(AdminUserOrderBy::Id),
      "username" => Some(AdminUserOrderBy::Username),
      "role" => Some(AdminUserOrderBy::Role),
      "createdAt" => Some(AdminUserOrderBy::CreatedAt),
      _ => None,
    }
  }

  fn filter_to_select(f: AdminUserFilter) -> SelectAdminUser {
    let role_eq = f.role_eq.and_then(|s|{
      match s.as_str() {
        "Admin" => Some(AdminRole::Admin),
        "SuperAdmin" => Some(AdminRole::SuperAdmin),
        _ => None,
      }
    });
    SelectAdminUser{
      username_ilike: into_like_search(f.username_like),
      role_eq,
      ..Default::default()
    }
  }

  async fn db_to_graphql(d: admin_user::AdminUser ) -> MyResult<Self> {
    Ok(AdminUser {
      id: d.attrs.id,
      username: d.attrs.username,
      hashed_password: d.attrs.hashed_password,
      otp_seed: d.attrs.otp_seed,
      role: d.attrs.role,
      created_at: d.attrs.created_at,
    })
  }
}

impl AdminUser {
  pub async fn create_admin_user(
    context: &Context, username: String, password: String, role: String
  ) -> FieldResult<AdminUser> {
    if context.role != AdminRole::SuperAdmin {
      return Err(field_error("401", "you don't have permission and you tried to hack the UI"));
    }
    let enum_role = match role.as_str() {
      "SuperAdmin" => AdminRole::SuperAdmin,
      "Admin" => AdminRole::Admin,
      _ => AdminRole::Admin,
    };

    let db_admin_user = context.site.admin_user().create(&username, &password, enum_role).await?;

    Ok(AdminUser::db_to_graphql(db_admin_user).await?)
  }

  pub async fn update_admin_user(
    context: &Context, password: String, otp: String, new_password: String
  ) -> FieldResult<AdminUser> {
    let db_admin_user = context.site.admin_user().find(&context.id).await?;

    if !db_admin_user.verify_password(&password) {
      return Err(field_error("wrong password", "Some data was wrong"));
    }
    if !db_admin_user.verify_otp(&otp) {
      return Err(field_error("wrong otp", "Some data was wrong"));
    }
  
    let admin_user = db_admin_user.update().hashed_password(hash_pass(&new_password)).save().await?;

    Ok(AdminUser::db_to_graphql(admin_user).await?)
  }
}