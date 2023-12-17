/*
 * A Bulletin is an enum that has variants for all possible bulletin states.
 * State transitions are enforced by making the methods only available to each variant.
 */

use super::*;
use bitcoin::{
  consensus, hashes::hex::FromHex, util::psbt::serialize::Serialize as PsbtSerialize, BlockHash,
  Transaction, Txid, network::constants::Network,
};
use duplicate::duplicate_item;
use num_traits::ToPrimitive;

model!{
  state: Site,
  table: bulletins,
  struct Bulletin {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(varchar)]
    #[serde(skip_serializing)]
    state: String,
    #[sqlx_model_hints(timestamptz)]
    started_at: UtcDateTime,
    #[sqlx_model_hints(timestamptz, default)]
    #[serde(skip_serializing)]
    submitted_at: Option<UtcDateTime>,
    #[sqlx_model_hints(varchar, default)]
    hash: Option<String>,
    #[sqlx_model_hints(varchar, default)]
    transaction: Option<String>,
    #[sqlx_model_hints(varchar, default)]
    transaction_hash: Option<String>,
    #[sqlx_model_hints(varchar, default)]
    block_hash: Option<String>,
    #[sqlx_model_hints(timestamptz, default)]
    block_time: Option<UtcDateTime>,
  },
  has_many {
    Document(bulletin_id)
  }
}

impl BulletinHub {
  /* Work is sequentially performed on one bulletin at a time.
   * first: waiting until submitted bulletins are published,
   * then: submitting new bulletins,
   * finally: rounding up the current draft into a new proposal.
   */
  pub async fn current(&self) -> sqlx::Result<Flow> {
    let row = self.state.db.fetch_one(sqlx::query!(r#"
      (SELECT id as "id!" FROM bulletins WHERE state != 'published' ORDER BY "id!" LIMIT 1)
      UNION ( SELECT current_draft() as "id!")
    "#)).await?;

    Ok(self.find(&row.id).await?.flow())
  }

  pub async fn current_draft(&self) -> sqlx::Result<(sqlx::Transaction<'static, sqlx::Postgres>, Draft)> {
    let tx = self.state.db.pool.begin().await?;

    let row = sqlx::query!(r#"SELECT current_draft() as "id!" FOR UPDATE"#)
      .fetch_one(&self.state.db.pool).await?;

    let draft = self.find(&row.id).await?.in_draft().expect("We know it's a draft");

    Ok((tx, draft))
  }

  /* This method was used to migrate populating block times for
   * bulletins that didn't have them */
  pub async fn populate_block_times(&self) -> ConstataResult<()> {
    use chrono::NaiveDateTime;
    use bitcoincore_rpc::{Auth, Client, RpcApi};
    let client = Client::new(
      self.state.settings.bitcoin_rpc_uri.clone(),
      Auth::UserPass(self.state.settings.bitcoin_rpc_user.clone(), self.state.settings.bitcoin_rpc_pass.clone()),
    )?;
    let empties = self.state.bulletin().select().state_eq(&"published".into()).block_time_is_set(false).all().await?;

    for e in empties {
      let block_time_seconds = client.get_block_info(&e.in_published()?.block_hash().into())?.time;
      let naive_datetime = NaiveDateTime::from_timestamp_opt(block_time_seconds.try_into().expect("not_too_big_epoch"), 0).expect("naive_datetime");
      let block_time: DateTime<Utc> = DateTime::from_utc(naive_datetime, Utc);
      e.update().block_time(Some(block_time)).save().await?;
    }
    Ok(())
  }
}

impl Bulletin {
  pub async fn payload(&self) -> ConstataResult<String> {
    let rows: Vec<String> = self.state.db.fetch_all_scalar(sqlx::query_scalar!(
      r#"(SELECT DISTINCT dp.hash as "hash!" FROM document_parts dp
        INNER JOIN documents d ON d.id = dp.document_id AND d.bulletin_id = $1)
       UNION
       (SELECT DISTINCT hash as "hash!" FROM pubkeys WHERE bulletin_id = $1)
       UNION
       (SELECT DISTINCT signature_hash as "hash!" FROM pubkeys WHERE bulletin_id = $1)
       UNION
       (SELECT DISTINCT signature_hash as "hash!" FROM document_part_signatures WHERE bulletin_id = $1)

       UNION
       (SELECT DISTINCT hash as "hash!" FROM story_snapshots WHERE bulletin_id = $1)
       UNION
       (SELECT DISTINCT evidence_hash as "hash!" FROM pubkey_domain_endorsements WHERE bulletin_id = $1)
       UNION
       (SELECT DISTINCT evidence_hash as "hash!" FROM email_addresses WHERE bulletin_id = $1)
       UNION
       (SELECT DISTINCT hash as "hash!" FROM terms_acceptances WHERE bulletin_id = $1)

       ORDER BY "hash!"
       "#,
      self.id()
    )).await?;

    let payload = rows.iter().fold(String::new(), |mut acc, i| {
      acc.push_str(i);
      acc.push('\n');
      acc
    });

    Ok(payload)
  }

  pub async fn mempool_info_url(&self) -> Option<String> {
    if let Some(transaction_hash) = &self.attrs.transaction_hash {
      let url = match self.state.settings.network {
        Network::Testnet => Some(format!("https://mempool.space/es/testnet/tx/{}", transaction_hash)),
        Network::Bitcoin => Some(format!("https://mempool.space/es/tx/{}", transaction_hash)),
        _ => None,
      };
      return url;
    }
    return None;
  }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(tag = "state")]
pub enum Flow {
  // It has a sequential ID, but documents may still be added to it.
  // There's always a current Draft.  All documents are required to point to a Draft.
  // Only one Draft can exist at a given time.
  Draft(Draft),

  // Bulletin that already knows which Documents it's going to stamp.
  // No documents can be added or removed, but it may revert to this state
  // if for whatever reason a blockchain transaction cannot be created for it.
  // Only one bulletin can be 'Proposed' or 'Submitted' at a given time.
  Proposed(Proposed),

  // These bulletins have an unconfirmed transaction in them.
  // If it does not confirm, it may be re-submitted.
  Submitted(Submitted),

  // These bulletins have been confirmed.
  Published(Published),
}

#[duplicate_item(flow_variant; [ Draft ]; [ Proposed ]; [ Submitted ]; [ Published ];)]
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct flow_variant(Bulletin);

#[duplicate_item(flow_variant; [ Draft ]; [ Proposed ]; [ Submitted ]; [ Published ];)]
impl flow_variant {
  pub fn id(&self) -> &i32 { self.0.id() }
  pub fn started_at(&self) -> &UtcDateTime { self.0.started_at() }
  pub fn submitted_at(&self) -> &Option<UtcDateTime> { self.0.submitted_at() }
  pub fn into_inner(self) -> Bulletin { self.0 }
  pub fn as_inner<'a>(&'a self) -> &'a Bulletin { &self.0 }
  pub async fn payload(&self) -> ConstataResult<String> { self.0.payload().await }
}

impl Bulletin {
  pub fn flow(&self) -> Flow {
    match self.state().as_ref() {
      "proposed" => Flow::Proposed(Proposed(self.clone())),
      "submitted" => Flow::Submitted(Submitted(self.clone())),
      "published" => Flow::Published(Published(self.clone())),
      _ => Flow::Draft(Draft(self.clone())),
    }
  }
}

#[duplicate_item(
  in_state          is_state          state_str       state_struct;
  [ in_draft      ] [ is_draft      ] [ "draft"     ] [ Draft     ];
  [ in_proposed   ] [ is_proposed   ] [ "proposed"  ] [ Proposed  ];
  [ in_submitted  ] [ is_submitted  ] [ "submitted" ] [ Submitted ];
  [ in_published  ] [ is_published  ] [ "published" ] [ Published ];
)]
impl Bulletin {
  pub fn in_state(&self) -> ConstataResult<state_struct> {
    self.flow().in_state()
  }

  pub fn is_state(&self) -> bool {
    self.attrs.state.as_str() == state_str
  }
}

#[duplicate_item(
  in_state          is_state          variant(i)             state_struct;
  [ in_draft      ] [ is_draft      ] [ Flow::Draft(i)     ] [ Draft     ];
  [ in_proposed   ] [ is_proposed   ] [ Flow::Proposed(i)  ] [ Proposed  ];
  [ in_submitted  ] [ is_submitted  ] [ Flow::Submitted(i) ] [ Submitted ];
  [ in_published  ] [ is_published  ] [ Flow::Published(i) ] [ Published ];
)]
impl Flow {
  pub fn in_state(&self) -> ConstataResult<state_struct> {
    if let variant([inner]) = self {
      Ok(inner.clone())
    } else {
      Err(Error::InvalidFlowState)
    }
  }

  pub fn is_state(&self) -> bool { matches!(self, variant([_])) }
}

impl Flow {
  pub fn as_inner<'a>(&'a self) -> &'a Bulletin {
    match self {
      Flow::Draft(a) => a.as_inner(),
      Flow::Proposed(a) => a.as_inner(),
      Flow::Submitted(a) => a.as_inner(),
      Flow::Published(a) => a.as_inner(),
    }
  }
}

#[duplicate_item(flow_variant; [ Proposed ]; [ Submitted ]; [ Published ];)]
impl flow_variant {
  pub fn hash(&self) -> &String { self.0.hash().as_ref().expect("Hash to be present at this stage") }
}

#[duplicate_item(flow_variant; [ Submitted ]; [ Published ];)]
impl flow_variant {
  pub fn raw_transaction(&self) -> &String {
    self.0.transaction().as_ref().expect("Transaction to be present at this stage")
  }

  pub fn raw_transaction_hash(&self) -> &String {
    self.0.transaction_hash().as_ref().expect("Transaction hash to be present at this stage")
  }

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
    Txid::from_hex(&self.raw_transaction_hash()).expect("Bulletin {} txid parse failed")
  }
  
  pub async fn bump_count(&self) -> sqlx::Result<i32> {
    self.0.state.bump()
      .select().bulletin_id_eq(self.id())
      .count().await
      .map(|i| i.to_i32().unwrap_or(0))
  }

  pub async fn last_bump(&self) -> sqlx::Result<Bump> {
    self.0.state.bump().select()
      .bulletin_id_eq(self.id())
      .order_by(BumpOrderBy::Counter)
      .desc(true)
      .one().await
  }
}

impl Draft {
  pub async fn propose(self) -> ConstataResult<Proposed> {
    /* We first set the bulletin to proposed then calculate the payload hash
     * to make sure we don't have a race condition and no new documents are
     * added to the current draft invalidating the payload hash.
     * This could probably be implemented with postgress locking instead,
     * and it's worth looking into doing it that way
     */

    let payload = self.0.payload().await?;

    if payload.is_empty() {
      return Err(crate::Error::validation( "payload", "cannot_propose_empty_draft",));
    }

    /* Two queries to ensure no race conditions happen.
     * We need a better locking mechanism here.
     */
    let updated = self.0
      .update()
      .state("proposed".to_string())
      .save().await?
      .update()
      .hash(Some(crate::models::hasher::hexdigest(&payload.as_bytes())))
      .save()
      .await?;

    Ok(Proposed(updated))
  }
}

impl Proposed {
  pub async fn submit(self, transaction: &Transaction) -> ConstataResult<Submitted> {
    let updated = self.0
      .update()
      .state("submitted".to_string())
      .submitted_at(Some(Utc::now()))
      .transaction(Some(hex::encode(PsbtSerialize::serialize(transaction))))
      .transaction_hash(Some(transaction.txid().to_string()))
      .save().await?;

    Ok(Submitted(updated))
  }
}

impl Submitted {
  pub async fn publish(self, block_hash: &BlockHash, block_time_seconds: u64) -> ConstataResult<Published> {
    use chrono::NaiveDateTime;
    let naive_datetime = NaiveDateTime::from_timestamp_opt(block_time_seconds.try_into().expect("not_too_big_epoch"), 0).expect("naive_datetime");
    let block_time: DateTime<Utc> = DateTime::from_utc(naive_datetime, Utc);

    let updated = self.0
      .update()
      .state("published".to_string())
      .block_hash(Some(block_hash.to_string()))
      .block_time(Some(block_time))
      .save().await?;

    for doc in updated.document_vec().await? {
      if let Some(att) = doc.story().await?.attestation().await? {
        att.on_done().await?;
      }
    }

    Ok(Published(updated))
  }

  pub async fn needs_bump(&self) -> ConstataResult<bool> {
    let bump_count = self.bump_count().await?;
    let bump_interval = self.0.state.settings.bump_interval();
    if bump_count >= 2 {
      return Ok(false);
    };

    let last_movement_date = if bump_count == 0 {
      *self.submitted_at()
    } else {
      Some(*self.last_bump().await?.started_at())
    };

    if let Some(date) = last_movement_date {
      if Utc::now() - date > bump_interval {
        return Ok(true);
      };
    }
    return Ok(false);
  }
  
  pub async fn create_bump(self, transaction: &Transaction) -> ConstataResult<Bump> {
    Ok(self.0.state.bump().insert(InsertBump {
        bulletin_id: self.id().clone(),
        started_at: Utc::now(),
        counter: self.bump_count().await? + 1,
        raw_transaction: hex::encode(PsbtSerialize::serialize(transaction)),
        raw_transaction_hash: transaction.txid().to_string(),
      })
      .save().await?
    )
  }

  /* Resubmitting a block is a critical admin task.
   * It's unlikely you want to use this method in your code.
   * This method will force override the transaction hash of any Bulletin.
   * If this method is called and there's a race condition we would have to
   * update the database with the correct TX information from the blockchain. BY HAND.
   */
  pub async fn resubmit(self, transaction: &Transaction) -> ConstataResult<Submitted> {
    Proposed(self.0).submit(transaction).await
  }
}

impl Published {
  pub fn block_time(&self) -> &UtcDateTime {
    self.0.block_time().as_ref()
      .expect("Published bulletin should have block time")
  }
  pub fn block_hash(&self) -> BlockHash {
    BlockHash::from_hex(
      self.0.block_hash().as_ref().expect("Published bulletin should have block hash")
    ).expect("Bulletin block hash parse failed")
  }
}

describe! {
  use bitcoin::{consensus::deserialize, hashes::hex::FromHex, Block, Transaction};
  use chrono::Utc;

  dbtest!{ can_fetch_a_locked_current_bulletin (site, _c)
    let (_tx, bulletin) = site.bulletin().current_draft().await?;
    assert_eq!(*bulletin.id(), 1);
  }

  regtest!{ can_timestamp_in_batches (site, c, chain)
    chain.fund_signer_wallet();
    let alice = c.alice().await.add_funds().await;

    alice.stories_with_signed_docs(b"").await;

    let first = site.bulletin().current().await?;
    assert_eq!(first.as_inner().id(), &1);

    assert_bulletin_payload(first.as_inner(), 12, vec![
      "2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae",
      "3f4df96aa420f500e9921fac125c16e0eee93dae08df141c1bed689c54d67de9",
      "433c4be47673b8c64771aba6cf6749a3de6ebbab52cca5ecb7913b26c3f72875",
      "6a41766726d979174ecfac83ca188d65ac9d3dd4f0e05d942dd68e072b8434db",
      "baa5a0964d3320fbc0c6a922140453c8513ea24ab8fd0577034804a967248096",
      "bb0f36c8bb2dd485dec356cc5fab4a207478e43ff6dac1d37519ac2cb825fdd3",
      "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
      "ec69f04bf69c2242741ca32fafdced0191033878b24c19d181505a8a107c892b",
      "fcde2b2edba56bf408601fb721fe9b5c338d10ee429ea04fae5511b68fbf8fb9",
    ]).await;

    // Current bulletin is the same.
    assert_that!(&first, eq(site.bulletin().current().await?));

    alice.stories_with_signed_docs(b"more_").await;

    // New documents are added to the same draft.
    assert_bulletin_payload(first.as_inner(), 21, vec![
      "2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae",
      "3f4df96aa420f500e9921fac125c16e0eee93dae08df141c1bed689c54d67de9",
      "433c4be47673b8c64771aba6cf6749a3de6ebbab52cca5ecb7913b26c3f72875",
      "6a41766726d979174ecfac83ca188d65ac9d3dd4f0e05d942dd68e072b8434db",
      "7e88f36da091492f222bf432ff5a85f172112b571ca17bd33c8f7f0392aca782",
      "90b657ac188b997d8b457a70d5e943d433113135a57456e692d8fb4f31517e42",
      "97e825796fcbf0968920112e54e728ef86604f5d0da29855ca5dae058fec439f",
      "b1e36323ab9112ef96d77f9c1f1de22244dc649cdbe1d91a5670b9d4a97dbcfa",
      "b69b7037a41b49579542747c9079752ed837746f8873f4b337200dcc59dc2812",
      "b9768f675585b74a98d9df051a53e5048b9b9b1f0e2e21565ba4f3e0ea22168c",
      "baa5a0964d3320fbc0c6a922140453c8513ea24ab8fd0577034804a967248096",
      "bb0f36c8bb2dd485dec356cc5fab4a207478e43ff6dac1d37519ac2cb825fdd3",
      "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
      "ec69f04bf69c2242741ca32fafdced0191033878b24c19d181505a8a107c892b",
      "fcde2b2edba56bf408601fb721fe9b5c338d10ee429ea04fae5511b68fbf8fb9",
    ]).await;

    let proposed = first.in_draft()?.propose().await?;
    assert_bulletin_payload(&proposed.as_inner(), 21, vec![
      "2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae",
      "3f4df96aa420f500e9921fac125c16e0eee93dae08df141c1bed689c54d67de9",
      "433c4be47673b8c64771aba6cf6749a3de6ebbab52cca5ecb7913b26c3f72875",
      "6a41766726d979174ecfac83ca188d65ac9d3dd4f0e05d942dd68e072b8434db",
      "7e88f36da091492f222bf432ff5a85f172112b571ca17bd33c8f7f0392aca782",
      "90b657ac188b997d8b457a70d5e943d433113135a57456e692d8fb4f31517e42",
      "97e825796fcbf0968920112e54e728ef86604f5d0da29855ca5dae058fec439f",
      "b1e36323ab9112ef96d77f9c1f1de22244dc649cdbe1d91a5670b9d4a97dbcfa",
      "b69b7037a41b49579542747c9079752ed837746f8873f4b337200dcc59dc2812",
      "b9768f675585b74a98d9df051a53e5048b9b9b1f0e2e21565ba4f3e0ea22168c",
      "baa5a0964d3320fbc0c6a922140453c8513ea24ab8fd0577034804a967248096",
      "bb0f36c8bb2dd485dec356cc5fab4a207478e43ff6dac1d37519ac2cb825fdd3",
      "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
      "ec69f04bf69c2242741ca32fafdced0191033878b24c19d181505a8a107c892b",
      "fcde2b2edba56bf408601fb721fe9b5c338d10ee429ea04fae5511b68fbf8fb9",
    ]).await;

    assert_that!(&proposed.hash(), rematch("[a-f0-9]{64}"));

    let submitted = proposed.submit(&transaction()).await?;

    assert_that!(
      &submitted.txid().to_string(),
      eq("f5864806e3565c34d1b41e716f72609d00b55ea5eac5b924c9719a842ef42206".into())
    );

    assert_that!(
      &submitted.transaction().txid().to_string(),
      eq(submitted.txid().to_string())
    );

    assert_that!(
      &submitted,
      eq(site.bulletin().current().await?.in_submitted()?)
    );

    let published = submitted.publish(&block().block_hash(), block().header.time as u64).await.unwrap();

    assert_eq!(published.block_time(), &Utc.with_ymd_and_hms(2009, 01, 14, 20, 40, 55).unwrap());
    assert_that!(
      &published.block_hash().to_string(),
      eq("00000000b0c5a240b2a61d2e75692224efd4cbecdf6eaf4cc2cf477ca7c270e7".into())
    );

    assert_eq!(site.bulletin().current().await?.as_inner().attrs.id, 2);
  }

  dbtest!{ it_does_not_repeat_hashes_in_the_same_bulletin (site, c)
    let message = &b"Hello World!"[..];
    let doc1 = c.alice().await.add_funds().await.signed_document(&message).await;
    let doc2 = c.bob().await.add_funds().await.signed_document(&message).await;

    assert_eq!(doc1.document_part_vec().await?[0].hash(), doc2.document_part_vec().await?[0].hash());
    assert_eq!(doc1.bulletin_id(), doc2.bulletin_id());

    let first = site.bulletin().current().await?;

    assert_bulletin_payload(&first.as_inner(), 10, vec![
      "0091641cb210e862b26ff551fece068a02b604a075fc6fd123bb2672f5b7562d",
      "347fdfa2e3ac333b2d36a703c6906a16899b9c9abe583d730ea884006abfd525",
      "3f4df96aa420f500e9921fac125c16e0eee93dae08df141c1bed689c54d67de9",
      "6a41766726d979174ecfac83ca188d65ac9d3dd4f0e05d942dd68e072b8434db",
      "7212717881098d39375358670512fda7c1c2e3a7f4ac1669933676cc11b21392",
      "7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069",
      "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
      "f5b6436a657fa4fd8bd8fb9d86eb4bc54b6c778a5668416a1087218cefa4bfa1",
    ]).await;
  }

  dbtest!{ it_can_resubmit_a_bulletin (site, c)
    let message = &b"Hello World!"[..];
    let _doc = c.alice().await.add_funds().await.signed_document(&message).await;

    let submitted = site.bulletin().current().await?
      .in_draft()?
      .propose().await?
      .submit(&transaction()).await?;

    let new_tx = deserialize(&Vec::<u8>::from_hex(
      "\
        020000000128a39da0c5eb71f25ede66e46175f3dd00e0321f96a8028f09f13446\
        1d40620f0000000000ffffffff022202000000000000086a060001000100019cd7\
        052a01000000160014849573e4735d7bbc6bd5fe9ece15544fd93a792900000000\
      ",
    )?)?;

    assert_eq!(
      submitted.raw_transaction_hash(),
      "f5864806e3565c34d1b41e716f72609d00b55ea5eac5b924c9719a842ef42206"
    );
    submitted.clone().resubmit(&new_tx).await.unwrap();

    let reloaded = site.bulletin().current().await?.in_submitted()?;
    assert_eq!(submitted.id(), reloaded.id());

    assert_eq!(
      reloaded.raw_transaction_hash(),
      "188075761b2a8561b98731a560eb03a7c1ae7a88a7b6b541819ac8770655e346"
    );
  }

  dbtest!{ it_cannot_propose_empty_bulletins (site, _c)
    let draft = site.bulletin().current().await?.in_draft()?;
    assert_that!(
      &draft.propose().await.unwrap_err(),
      structure!{ crate::Error::Validation{ message: eq("cannot_propose_empty_draft".to_string()) } }
    );
  }

  pub fn transaction() -> Transaction {
    let hex_tx = Vec::<u8>::from_hex(
      "\
      02000000000101595895ea20179de87052b4046dfe6fd515860505d6511a9004\
      cf12a1f93cac7c0100000000ffffffff01deb807000000000017a9140f3444e2\
      71620c736808aa7b33e370bd87cb5a078702483045022100fb60dad8df4af284\
      1adc0346638c16d0b8035f5e3f3753b88db122e70c79f9370220756e6633b17f\
      d2710e626347d28d60b0a2d6cbb41de51740644b9fb3ba7751040121028fa937\
      ca8cba2197a37c007176ed8941055d3bcb8627d085e94553e62f057dcc00000000\
    ",
    )
    .unwrap();
    deserialize(&hex_tx).unwrap()
  }

  pub fn block() -> Block {
    let hex_block = Vec::<u8>::from_hex(
      "\
      010000004ddccd549d28f385ab457e98d1b11ce80bfea2c5ab93015ade4973e40\
      0000000bf4473e53794beae34e64fccc471dace6ae544180816f89591894e0f41\
      7a914cd74d6e49ffff001d323b3a7b02010000000100000000000000000000000\
      00000000000000000000000000000000000000000ffffffff0804ffff001d026e\
      04ffffffff0100f2052a0100000043410446ef0102d1ec5240f0d061a4246c1bd\
      ef63fc3dbab7733052fbbf0ecd8f41fc26bf049ebb4f9527f374280259e7cfa99\
      c48b0e3f39c51347a19a5819651503a5ac00000000010000000321f75f3139a01\
      3f50f315b23b0c9a2b6eac31e2bec98e5891c9246648899422600000000494830\
      45022100cb2c6b346a978ab8c61b18b5e9397755cbd17d6eb2fe0083ef32e067f\
      a6c785a02206ce44e613f31d9a6b0517e46f3db1576e9812cc98d159bfdaf759a\
      5014081b5c01ffffffff79cda0945903627c3da1f85fc95d0b8ee3e76ae0cfdc9\
      a65d09744b1f8fc85430000000049483045022047957cdd957cfd0becd642f6b8\
      4d82f49b6cb4c51a91f49246908af7c3cfdf4a022100e96b46621f1bffcf5ea59\
      82f88cef651e9354f5791602369bf5a82a6cd61a62501fffffffffe09f5fe3ffb\
      f5ee97a54eb5e5069e9da6b4856ee86fc52938c2f979b0f38e820000000048473\
      04402204165be9a4cbab8049e1af9723b96199bfd3e85f44c6b4c0177e3962686\
      b26073022028f638da23fc003760861ad481ead4099312c60030d4cb57820ce4d\
      33812a5ce01ffffffff01009d966b01000000434104ea1feff861b51fe3f5f8a3\
      b12d0f4712db80e919548a80839fc47c6a21e66d957e9c5d8cd108c7a2d2324ba\
      d71f9904ac0ae7336507d785b17a2c115e427a32fac00000000\
    ",
    )
    .unwrap();
    deserialize(&hex_block).unwrap()
  }
}
