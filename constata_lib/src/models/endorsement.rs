use super::*;

#[derive(PartialEq, Clone, Debug, Serialize)]
#[serde(tag = "type")]
pub enum Endorsement {
  Website { url: String },
  EmailAddress { address: String, keep_private: bool },
  Kyc { attrs: super::kyc_endorsement::KycEndorsementAttrs, },
  Telegram { attrs: super::telegram::TelegramUserAttrs, },
}
