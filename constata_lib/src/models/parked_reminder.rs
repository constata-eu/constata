use super::*;
use chrono::{Datelike, Timelike, Duration, Weekday};

model!{
  state: Site,
  table: parked_reminders,
  struct ParkedReminder {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    org_id: i32,
    #[sqlx_model_hints(varchar)]
    address: String,
    #[sqlx_model_hints(timestamptz, default)]
    created_at: UtcDateTime,
    #[sqlx_model_hints(timestamptz)]
    sent_at: Option<UtcDateTime>,
  },
  belongs_to {
    Org(org_id),
  }
}

impl ParkedReminderHub {
  pub async fn create_new_reminders(&self, now: UtcDateTime) -> ConstataResult<()> {
    if !self.is_good_time_for_reminders(now).await? {
      return Ok(());
    }

    let orgs = self.state.org()
      .has_two_day_old_parkeds_and_email_and_no_recent_reminders(now)
      .all().await?;

    for org in orgs {
      self.create(org).await?;
    }

    Ok(())
  }

  pub async fn is_good_time_for_reminders(&self, now: UtcDateTime) -> ConstataResult<bool> {
    let time = now.time();
    if time.hour() != 13 {
      return Ok(false);
    }
    let previous = self.last_reminder_date(now).await?;
    let over_four_days = now - previous > Duration::days(4);
    let over_a_week = now - previous > Duration::days(7);

    return Ok((over_four_days && now.weekday() == Weekday::Mon) || over_a_week)
  }

  pub async fn last_reminder_date(&self, now: UtcDateTime) -> ConstataResult<UtcDateTime> {
    Ok(self.state.parked_reminder()
      .select()
      .order_by(ParkedReminderOrderBy::CreatedAt)
      .desc(true)
      .optional().await? 
      .map(|r| r.attrs.created_at )
      .unwrap_or_else(|| now - Duration::days(8)))
  }

  pub async fn not_sent(&self) -> sqlx::Result<Vec<ParkedReminder>> {
    self.select().sent_at_is_set(false).all().await
  }

  pub async fn create(&self, org: Org) -> sqlx::Result<ParkedReminder> {
    self.insert(InsertParkedReminder{
      org_id: org.attrs.id,
      address: org.billing().await?.last_email_address().await?.attrs.address,
      sent_at: None,
    }).save().await
  }
}


impl ParkedReminder {
  pub async fn render_parked_mailer_html(&self) -> ConstataResult<String> {
    EmailParkedDocuments::new(self).await?.render_html()
  }

  pub async fn mark_sent(self) -> ConstataResult<ParkedReminder> {
    if self.sent_at().is_some() {
      return Err(Error::validation("sent_at", "cannot_mark_as_sent"));
    }

    Ok(self.update().sent_at(Some(Utc::now())).save().await?)
  }
}
