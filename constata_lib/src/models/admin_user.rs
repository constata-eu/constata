use super::*;
use juniper::GraphQLEnum;
use google_authenticator::GoogleAuthenticator;

#[derive(sqlx::Type, Copy, Clone, Debug, Deserialize, PartialEq, Serialize, GraphQLEnum)]
#[sqlx(type_name = "admin_role", rename_all = "lowercase")]
pub enum AdminRole {
  SuperAdmin,
  Admin,
}

impl sqlx::postgres::PgHasArrayType for AdminRole {
  fn array_type_info() -> sqlx::postgres::PgTypeInfo {
    sqlx::postgres::PgTypeInfo::with_name("_admin_role")
  }
}

model!{
  state: Site,
  table: admin_users,
  struct AdminUser {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(varchar)]
    username: String,
    #[sqlx_model_hints(varchar)]
    hashed_password: String,
    #[sqlx_model_hints(varchar)]
    otp_seed: String,
    #[sqlx_model_hints(admin_role)]
    role: AdminRole,
    created_at: UtcDateTime,
  }
}

impl AdminUserHub {
  pub async fn create(&self, username: &str, password: &str, role: AdminRole) -> sqlx::Result<AdminUser> {
    self.insert(InsertAdminUser{
      username: username.to_string(),
      hashed_password: hash_pass(password),
      otp_seed: GoogleAuthenticator::new().create_secret(32),
      role,
      created_at: Utc::now(),
    }).save().await
  }

  pub async fn find_from_credentials(&self, username: &str, password: &str) -> sqlx::Result<AdminUser> {
    self.select().username_eq(&username.into()).password_eq(password).one().await
  }

  pub async fn find_from_id(&self, id: &i32, password: &str) -> sqlx::Result<AdminUser> {
    self.select().id_eq(id).password_eq(password).one().await
  }
}

impl SelectAdminUserHub {
  pub fn password_eq(self, password: &str) -> SelectAdminUserHub {
    self.hashed_password_eq(&hash_pass(password))
  }
}

impl AdminUser {
  pub fn verify_otp(&self, otp: &str) -> bool {
    GoogleAuthenticator::new().verify_code(self.otp_seed(), otp, 6, 0) 
  }

  pub fn verify_password(&self, password: &str) -> bool {
    self.attrs.hashed_password ==  hash_pass(password)
  }

  pub fn get_current_otp(&self) -> ConstataResult<String> {
    GoogleAuthenticator::new().get_code(self.otp_seed(),0)
      .map_err(|_| Error::validation("otp_seed", "cannot_generate_codes"))
  }
}

pub fn hash_pass(pass: &str) -> String {
  hasher::hexdigest(format!("constata{}", pass).as_bytes())
}

describe!{
  dbtest!{ creates_admin_user_and_validates_its_credentials(site, _c)
    let admin = site.admin_user().create("development@constata.eu", "password", AdminRole::SuperAdmin).await?;

    let from_creds = site.admin_user().find_from_credentials("development@constata.eu", "password").await?;
    assert_eq!(admin, from_creds);
    assert!(site.admin_user().find_from_credentials("development@constata.eu", "badpass").await.is_err());

    let otp = GoogleAuthenticator::new().get_code(admin.otp_seed(),0)?;
    assert!(from_creds.verify_otp(&otp));
    assert!(!from_creds.verify_otp("000000"));
    from_creds.update().otp_seed("GIVZS767V2UXZINNPJCBKQTO6JPPCT4N".to_string()).save().await?;
  }
}
