use crate::error::Result;
use super::*;

model!{
  state: Site,
  table: gifts,
  struct Gift {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    org_id: i32,
    created_at: UtcDateTime,
    #[sqlx_model_hints(decimal)]
    tokens: Decimal,
    reason: String,
  }
}

impl GiftHub {
  pub async fn give_and_trigger_updates(&self, org_id: i32, tokens: Decimal, reason: &str) -> Result<Gift> {
    self.build(org_id, tokens, reason).save_and_trigger_updates().await
  }

  pub fn build(&self, org_id: i32, tokens: Decimal, reason: &str) -> InsertGiftHub {
    self.insert(InsertGift{
      org_id,
      tokens,
      created_at: Utc::now(),
      reason: reason.to_string(),
    })
  }
}

impl InsertGiftHub {
  pub async fn save_and_trigger_updates(self) -> Result<Gift> {
    let gift = self.save().await?;
    gift.state.org()
      .find(gift.org_id()).await?
      .account_state().await?
      .fund_all_documents().await?;
    Ok(gift)
  }
}

