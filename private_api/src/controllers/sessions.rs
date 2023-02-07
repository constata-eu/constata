use super::*;
use constata_lib::{ models::admin_user::AdminRole, Error };
pub use serde::de::DeserializeOwned;

const DURATION: i64 = 60 * 72;

#[post("/", data="<form>")]
pub async fn create(site: &State<Site>, form: Json<SessionForm>) -> JsonResult<Session> {
  
  let admin = site.admin_user().find_from_credentials(&form.username, &form.password).await
    .map_err(|_| Error::Unauthorized)?;
    
  if !admin.verify_otp(&form.otp) {
    return Err(Error::Unauthorized);
  }

  let session = Session {
    username: admin.attrs.username,
    permissions: admin.attrs.role,
    token: site.admin_user_session().create(admin.attrs.id, DURATION).await?.attrs.token
  };

  Ok(Json(session))
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionForm {
  username: String,
  password: String,
  otp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
  username: String,
  permissions: AdminRole,
  token: String,
}


constata_lib::describe_one! {
  use google_authenticator::GoogleAuthenticator;
  use constata_lib::models::{
    admin_user::hash_pass,
    Utc,
  };
  
  apitest!{ login_admin_user (db, c, client)

    db.admin_user().create("foo", "barz", AdminRole::SuperAdmin).await?;
    let admin = db.admin_user().find_from_credentials("foo", "barz").await.unwrap();
    let otp = GoogleAuthenticator::new().get_code(&admin.attrs.otp_seed, 0)?;
    assert!(admin.verify_otp(&otp));

    let body = body_login_admin_user("foo", "barz", &otp);
    let token = client.post_for_session_and_get_token("/sessions/", body).await;

    let token_from_db = db.admin_user_session().find_active(&token, Utc::now())
                            .await.unwrap().unwrap()
                            .attrs.token;

    assert_eq!(token, token_from_db);
  }

  apitest!{ login_admin_user_bad_password (db, c, client)

    db.admin_user().create("foo", "barz", AdminRole::SuperAdmin).await?;
    let admin = db.admin_user().find_from_credentials("foo", "barz").await?;
    let otp = GoogleAuthenticator::new().get_code(&admin.attrs.otp_seed, 0)?;
    assert!(admin.verify_otp(&otp));

    let bad_password = "bad_password";
    let body = body_login_admin_user("foo", &bad_password, &otp);
    let response = client.post_for_session("/sessions/", body).await;
    assert!(response.status() != Status::Ok);

  }

  apitest!{ login_admin_user_bad_otp (db, c, client)

    db.admin_user().create("foo", "barz", AdminRole::SuperAdmin).await?;
    let bad_otp = "123456";
    let body = body_login_admin_user("foo", "barz", &bad_otp);
    let response = client.post_for_session("/sessions/", body).await;
    assert!(response.status() != Status::Ok);

  }

  apitest!{ private_graphql_correct_authorization (db, c, client)

    let admin = db.admin_user().create("foo", "barz", AdminRole::SuperAdmin).await?;
    let session = db.admin_user_session().create(*admin.id(), 10).await?;
    assert_eq!(session.role().await?, AdminRole::SuperAdmin);

    let token = session.attrs.token;
    let body = body_search_admin_user(admin.attrs.id.clone());
    let response = client.post_with_token("/graphql/", token, body)
                      .await;

    assert_eq!(response, expected_for_correct_authorization(admin.attrs.id));  
  }

  apitest!{ private_graphql_incorrect_token (_db, c, client)

    let token = format!("bad_token");
    let body = body_search_admin_user(0);
    let response = client.post_with_token_raw("/graphql/", token, body).await;
    assert_eq!(response.status(), Status::Unauthorized);
  }

  apitest!{ private_graphql_no_token (_db, c, client)

    let body = body_search_admin_user(0);
    let response = client.post_with_no_token("/graphql/", body).await;
    assert_eq!(response.status(), Status::Unauthorized);
  }

  apitest!{ private_graphql_no_super_admin (db, c, client)

    let admin = db.admin_user().create("foo", "barz", AdminRole::Admin).await?;
    let session = db.admin_user_session().create(*admin.id(), 10).await?;
    assert_eq!(session.role().await?, AdminRole::Admin);

    let token = session.attrs.token;
    let body_search = body_search_admin_user(admin.attrs.id);
    let response_search = client.post_with_token("/graphql/", token.clone(), body_search).await;
    let message_search = response_search.split("\"").collect::<Vec<&str>>()[7];
    assert_eq!(message_search, "401");

    let body_create = body_create_admin_user("user", "password", "SuperAdmin");
    let response_create = client.post_with_token("/graphql/", token, body_create).await;
    let message_create = response_create.split("\"").collect::<Vec<&str>>()[7];
    assert_eq!(message_create, "401");
  }

  apitest!{ private_graphql_change_password (db, c, client)

    db.admin_user().create("foo", "barz", AdminRole::Admin).await?;
    let admin = db.admin_user().find_from_credentials("foo", "barz").await?;
    let token = client.login_and_get_token("foo", "barz").await;
    let otp = GoogleAuthenticator::new().get_code(&admin.attrs.otp_seed,0)?;

    let body = body_change_password("barz", &otp, "new_password");
    let response = client.post_with_token("/graphql/", token, body).await;
    assert_eq!(response, expected_for_change_password(admin.attrs.id, "new_password"));

    let check_new_hashed_pass = db.admin_user().find(admin.id()).await?.attrs.hashed_password;
    assert_eq!(check_new_hashed_pass, hash_pass("new_password"))
  }

  apitest!{ private_graphql_change_password_with_wrong_password (db, c, client)
    db.admin_user().create("foo", "barz", AdminRole::Admin).await?;
    let admin = db.admin_user().find_from_credentials("foo", "barz").await?;
    let token = client.login_and_get_token("foo", "barz").await;
    let otp = GoogleAuthenticator::new().get_code(&admin.attrs.otp_seed,0)?;

    let body = body_change_password("bad_password", &otp, "new_password");
    let response = client.post_with_token_message_graphql("/graphql/", token, body).await;
    assert_eq!(response, "wrong password");

    let no_new_hashed_pass = db.admin_user().find(admin.id()).await?.attrs.hashed_password;
    assert_eq!(no_new_hashed_pass, hash_pass("barz"))
  }

  apitest!{ private_graphql_change_password_with_wrong_otp (db, c, client)

    db.admin_user().create("foo", "barz", AdminRole::Admin).await?;
    let admin = db.admin_user().find_from_credentials("foo", "barz").await?;
    let token = client.login_and_get_token("foo", "barz").await;

    let body = body_change_password("barz", "bad_otp", "new_password");
    let response = client.post_with_token_message_graphql("/graphql/", token, body).await;
    assert_eq!(response, "wrong otp");

    let no_new_hashed_pass = db.admin_user().find(admin.id()).await?.attrs.hashed_password;
    assert_eq!(no_new_hashed_pass, hash_pass("barz"))
  }


  fn body_change_password(password: &str, otp: &str, new_password: &str) -> String {
    format!("mutation {{ 
      updateAdminUser(password: \"{}\", otp: \"{}\", newPassword: \"{}\")
      {{ id, username, hashedPassword }} }}",
      password, otp, new_password)
  }
  
  fn body_create_admin_user(username: &str, password: &str, role: &str) -> String {
    format!("mutation {{
      createAdminUser(username: \"{}\", password: \"{}\", role: \"{}\")
      {{ username }} }}",
      username, password, role)
  }
  
  fn body_search_admin_user(id: i32) -> String {
    format!("{{ AdminUser(id: {}) {{ id, username }} }}", id)
  }
  
  pub fn body_login_admin_user(username: &str, password: &str, otp: &str) -> String {
    serde_json::json![{
      "username": username,
      "password": password,
      "otp": otp
    }].to_string()
  }
  
  pub fn expected_for_correct_authorization(id: i32) -> String {
    serde_json::json![{
      "data": {
          "AdminUser": {
            "id": id,
            "username": "foo",
          }}}].to_string()
  }
  
  pub fn expected_for_change_password(id: i32, new_password: &str) -> String {
    serde_json::json![{
      "data": {
          "updateAdminUser": {
            "id": id,
            "username": "foo",
            "hashedPassword": hash_pass(new_password)
          }}}].to_string()

  }
}
