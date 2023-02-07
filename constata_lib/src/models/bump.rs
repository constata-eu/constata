use crate::Site;
use super::*;
use bitcoin::{ consensus, hashes::hex::FromHex, Transaction, Txid };


model!{
  state: Site,
  table: bumps,
  struct Bump {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    bulletin_id: i32,
    #[sqlx_model_hints(timestamptz)]
    started_at: UtcDateTime,
    #[sqlx_model_hints(int4)]
    counter: i32,
    #[sqlx_model_hints(varchar)]
    raw_transaction: String,
    #[sqlx_model_hints(varchar)]
    raw_transaction_hash: String,
  }
}

impl Bump {

  pub fn transaction(&self) -> Transaction {
    let hex = Vec::<u8>::from_hex(&self.raw_transaction())
      .unwrap_or_else(|_| panic!("Bulletin {} has invalid transaction data", self.raw_transaction_hash()));

    let transaction: Transaction = consensus::deserialize(&hex).unwrap_or_else(|_| {
      panic!(
        "Bulletin {} transaction cannot be deserialized",
        self.raw_transaction_hash()
      )
    });

    assert_eq!(
      transaction.txid(),
      self.txid(),
      "Bulletin {} txid and payload don't match",
      self.raw_transaction_hash()
    );

    transaction
  }

  pub fn txid(&self) -> Txid {
    Txid::from_hex(&self.raw_transaction_hash()).expect("Bump {} txid parse failed")
  }
}