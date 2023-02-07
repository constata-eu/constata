use super::{*, admin_user::AdminRole};
use chrono::{Duration, Utc};
pub use rocket::{ http::Status, request::{FromRequest, Outcome, Request}, };

const RENEWAL_TIME_ALLOWED: i64 = 60 * 120;
const RENOVATION: i64 = 60 * 72;

model!{
  state: Site,
  table: admin_user_sessions,
  struct AdminUserSession {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    admin_user_id: i32,
    #[sqlx_model_hints(varchar)]
    token: String,
    #[sqlx_model_hints(boolean)]
    expired: bool,
    created_at: UtcDateTime,
    expires_at: UtcDateTime,
  }
}

impl AdminUserSessionHub {
  pub async fn create(&self, admin_user_id: i32, expires_in_minutes: i64) -> sqlx::Result<AdminUserSession> {
    self.insert(InsertAdminUserSession{
      admin_user_id,
      token: format!("{}-{}", admin_user_id, MagicLink::make_random_token()),
      expired: false,
      created_at: Utc::now(),
      expires_at: Utc::now() + Duration::minutes(expires_in_minutes.into()),
    }).save().await
  }

  pub async fn find_active(&self, token: &String, time_now: UtcDateTime) -> sqlx::Result<Option<AdminUserSession>> {
    let maybe_session = self.select()
      .token_eq(token)
      .expired_eq(&false)
      .optional().await?;

    let res = match maybe_session {
      None => None,
      Some(session) => {
        if session.expires_at() < &time_now {
          session.update().expired(true).save().await?;
          None
        } else if session.verify_renovation(&time_now).await { 
          let session = session.update()
            .expires_at(
                time_now + Duration::minutes(RENOVATION)
            ).save().await?;
          Some(session)
        } else {
          Some(session)
        }
      }
    };

    Ok(res)
  }
}

impl AdminUserSession {
  pub async fn role(&self) -> sqlx::Result<AdminRole> {
    Ok(self.state.admin_user().find(self.admin_user_id()).await?.attrs.role)
  }
  pub async fn verify_renovation(&self, time_now: &UtcDateTime) -> bool {
    if *self.created_at() + Duration::minutes(RENEWAL_TIME_ALLOWED) < *time_now {
      return false;
    }
    true
  }
}


#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminUserSession {
  type Error = ();

  async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    async fn build(req: &Request<'_>) -> Option<AdminUserSession> {
      let token = req.headers().get("Authorization").collect::<Vec<_>>().pop()?;
      let site = req.rocket().state::<Site>()?;
      let found = site.admin_user_session().find_active(&token.to_string(), Utc::now()).await.ok()?;
      found
    }

    match build(req).await {
      Some(session) => Outcome::Success(session),
      None => Outcome::Failure((Status::Unauthorized, ())),
    }
  }
}

describe!{
  use chrono::Duration;
  const DURATION: i64 = 60 * 72;

  dbtest!{ creates_a_new_user_session(site, _c)
    let admin = site.admin_user().create("foo", "barz", AdminRole::SuperAdmin).await?;
    let session = site.admin_user_session().create(*admin.id(), DURATION).await?;
    assert_eq!(session.role().await?, AdminRole::SuperAdmin);
    let found = site.admin_user_session().find_active(session.token(), Utc::now()).await?.unwrap();
    assert!(
      site.admin_user_session()
      .find_active(&"this-is-a-bogus-token".to_string(), Utc::now())
      .await?
      .is_none()
    );
    assert_eq!(&found.token(), &session.token());
  }

  dbtest!{ automatically_expires_session(site, _c)
    let admin = site.admin_user().create("foo", "barz", AdminRole::SuperAdmin).await?;
    let session = site.admin_user_session().create(*admin.id(), 0).await?;
    assert!(!session.expired());
    assert!(
      site.admin_user_session()
        .find_active(session.token(), Utc::now()).await?
        .is_none()
    );
    assert!(session.reloaded().await?.expired());
  }

  dbtest!{ automatically_renovates_session_until_expires(site, _c)
    
    let admin = site.admin_user().create("foo", "barz", AdminRole::SuperAdmin).await?;
    let session = site.admin_user_session().create(*admin.id(), DURATION).await?;
    let token = session.attrs.token.clone();
    let find_active = test_sessions(site.clone(), 1, &token).await.unwrap().unwrap();
    assert!(!find_active.expired());

    let session_two_days = test_sessions(site.clone(), 48, &token).await.unwrap().unwrap();
    assert!(session.attrs.expires_at < session_two_days.attrs.expires_at);

    let session_four_days = test_sessions(site.clone(), 96, &token).await.unwrap().unwrap();
    assert!(session_two_days.attrs.expires_at < session_four_days.attrs.expires_at);
  
    let session_five_days = test_sessions(site.clone(), 120, &token).await.unwrap().unwrap();
    assert_eq!(session_four_days.attrs.expires_at, session_five_days.attrs.expires_at);

    let session_seven_days = test_sessions(site.clone(), 168, &token).await.unwrap();
    assert!(session_seven_days.is_none());
    assert!(session.reloaded().await?.expired());
  }

  async fn test_sessions(site: Site, hours: i64, token: &str) -> sqlx::Result<Option<AdminUserSession>> {
    let date_of_request = Utc::now() + Duration::minutes(60 * hours);
    let session = site.admin_user_session().find_active(&token.to_string(), date_of_request).await;
    session
  }
}


