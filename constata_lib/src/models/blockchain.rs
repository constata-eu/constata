use super::{
  *,
  bulletin::{Flow, Draft, Proposed, Published, Submitted}
};
use bitcoin::{
  blockdata::{opcodes::all as opcodes, script::Builder},
  Address, Amount, BlockHash, OutPoint, Script, SigHashType, Transaction, TxIn, TxOut
};
pub use bitcoin::PrivateKey;
use bitcoin_wallet::account::{MasterAccount, Unlocker};
use bitcoincore_rpc::jsonrpc::error as rpc_error;
use bitcoincore_rpc::Error::JsonRpc;
use bitcoincore_rpc::{json::ListUnspentResultEntry, Auth, Client, RpcApi};
use bitcoincore_rpc::bitcoincore_rpc_json::EstimateMode;
use chrono::Utc;
use std::collections::HashMap;
use num_traits::ToPrimitive;

pub struct Blockchain {
  pub site: Site,
  pub master_account: MasterAccount,
  pub address: Address,
  pub unlocker: Unlocker,
  pub client: Client,
  pub default_fee: u64,
}

#[derive(Debug)]
pub struct Stats {
  pub address: Address,
  pub balance: u64,
  pub block_height: u64,
  pub block_hash: BlockHash,
  pub current_bulletin: Flow,
}

impl Blockchain {
  pub async fn from_site(site: Site) -> ConstataResult<Blockchain> {
    let keyring = site.keyring()?
      .ok_or_else(|| Error::Init("Cannot use blockchain with a site with no keyring".into()))?;

    let client = Client::new(
      site.settings.bitcoin_rpc_uri.clone(),
      Auth::UserPass(site.settings.bitcoin_rpc_user.clone(), site.settings.bitcoin_rpc_pass.clone()),
    )?;

    Ok(Self {
      site: site,
      master_account: keyring.master_account,
      unlocker: keyring.unlocker,
      address: keyring.address,
      client,
      default_fee: 30_000,
    })
  }

  pub async fn process(&mut self) -> ConstataResult<Flow> {
    match self.site.bulletin().current().await? {
      Flow::Draft(b) => {
        self.propose(b).await?;
      }
      Flow::Proposed(b) => {
        self.submit(b).await?;
      }
      Flow::Submitted(b) => {
        self.sync_submitted(b).await?;
      }
      _ => {} 
    }

    Ok(self.site.bulletin().current().await?)
  }

  pub async fn propose(&mut self, draft: Draft) -> ConstataResult<()> {
    if Utc::now() - *draft.started_at() > self.site.settings.minimum_bulletin_interval() {
      draft.propose().await?;
    }
    Ok(())
  }

  pub async fn submit(&mut self, proposed: Proposed) -> ConstataResult<Submitted> {
    let transaction = self.build_and_sign(
      &hex::decode(proposed.hash()).expect("Should never store non-hex in DB"),
      self.get_utxos(Some(1))?,
      self.sats_per_byte_economy()?,
    )?;

    let submitted = proposed.submit(&transaction).await?;

    self.client.send_raw_transaction(&transaction)?;

    Ok(submitted)
  }

  pub async fn sync_submitted(&mut self, submitted: Submitted) -> ConstataResult<Option<Published>> {
    let raw_result = self.client.get_transaction(&submitted.txid(), Some(true));

    // The RPC -5 error means the transaction was not propagated correctly before.
    if let Err(JsonRpc(rpc_error::Error::Rpc(rpc_error::RpcError { code: -5, .. }))) = raw_result {
      let _dontcare = self.client.send_raw_transaction(&submitted.transaction());
      return Ok(None);
    }

    let result = raw_result?;

    if result.info.confirmations >= 2 {
      return Ok(Some(
        submitted
          .publish(&result.info.blockhash.expect("blockhash"), result.info.blocktime.expect("blocktime"))
          .await?,
      ))
    } else if submitted.needs_bump().await? {
      self.bump_fee().await?;
      return Ok(None)
    } else {
      return Ok(None)
    };
  }

  pub async fn bump_fee(&mut self) -> ConstataResult<(Transaction, Submitted)> {
    let submitted = self.site.bulletin().current()
      .await?
      .in_submitted()
      .map_err(|_| Error::Stamping("no_current_submitted_bulletin".to_string()))?;

    let sats = self.sats_per_byte_fast()?;
    println!("Using sats per byte: {sats}");
    let calculated_sats = sats + (sats * (submitted.bump_count().await?.to_u64().unwrap_or(0) + 1) / 10);
    let utxos = self.get_utxos(None)?;
    let transaction = self.build_and_sign(b"bumpfee", utxos.clone(), calculated_sats)?;
    self.client.send_raw_transaction(&transaction)?;
    
    submitted.clone().create_bump(&transaction).await?;
    
    Ok((transaction, submitted))
  }

  pub async fn resubmit(&mut self) -> ConstataResult<(Transaction, Transaction, Submitted)> {
    let submitted = self.site.bulletin().current().await?.in_submitted()
      .map_err(|_| Error::Stamping("no_current_submitted_bulletin".to_string()))?;

    let old_transaction = submitted.transaction();

    let transaction = self.build_and_sign(
      &hex::decode(submitted.hash()).expect("Should never store non-hex in DB"),
      self.get_utxos(Some(1))?,
      self.sats_per_byte_economy()?,
    )?;

    let resubmitted = submitted.resubmit(&transaction).await?;

    self.client.send_raw_transaction(&transaction)?;

    Ok((old_transaction, transaction, resubmitted))
  }

  fn get_utxos(&self, confirmations: Option<usize>) -> ConstataResult<Vec<ListUnspentResultEntry>> {
    Ok(
      self
        .client
        .list_unspent(confirmations, None, Some(&[&self.address]), None, None)?,
    )
  }

  fn sats_per_byte(&self, number_block: u16) -> ConstataResult<u64> {
    let calculated = self
        .client
        .estimate_smart_fee(number_block, Some(EstimateMode::Economical))?
        .fee_rate
        .unwrap_or_else(|| Amount::from_sat(self.default_fee))
        .as_sat()
        / 1_000;

    Ok(calculated.max(2).min(30))
  }

  fn sats_per_byte_fast(&self) -> ConstataResult<u64> {
    self.sats_per_byte(3)
  }

  fn sats_per_byte_economy(&self) -> ConstataResult<u64> {
    self.sats_per_byte(6)
  }

  fn build_and_sign(
    &mut self,
    op_return: &[u8],
    raw_utxos: Vec<ListUnspentResultEntry>,
    sat_per_byte: u64,
  ) -> ConstataResult<Transaction> {
    let (mut transaction, prevouts) = self.build_transaction(op_return, raw_utxos, sat_per_byte)?;

    self.master_account.sign(
      &mut transaction,
      SigHashType::All,
      &(|outpoint| prevouts.get(&outpoint).cloned()),
      &mut self.unlocker,
    )?;

    Ok(transaction)
  }

  fn build_transaction(
    &self,
    op_return: &[u8],
    mut raw_utxos: Vec<ListUnspentResultEntry>,
    sat_per_byte: u64,
  ) -> ConstataResult<(Transaction, HashMap<OutPoint, TxOut>)> {
    let input_script_size = 139;
    let mut outputs = vec![TxOut {
      value: 546,
      script_pubkey: Builder::new()
        .push_opcode(opcodes::OP_RETURN)
        .push_slice(op_return)
        .into_script(),
    }];
    let spent_amount: u64 = outputs.iter().map(|o| o.value).sum();

    raw_utxos.sort_by_key(|r| r.txid);

    let mut utxos: Vec<ListUnspentResultEntry> = vec![];
    let mut total_amount = 0;
    let mut expected_size = ((outputs.len() + 1) * 34) as u64;
    for utxo in raw_utxos {
      total_amount += utxo.amount.as_sat();
      utxos.push(utxo);
      expected_size += input_script_size;
    }

    if total_amount < spent_amount {
      return Err(Error::Stamping("wallet_is_empty".to_string()));
    }

    let mut inputs = vec![];
    let mut prev_txouts = HashMap::new();

    for utxo in utxos {
      let outpoint = OutPoint {
        txid: utxo.txid,
        vout: utxo.vout,
      };
      inputs.push(TxIn {
        previous_output: outpoint,
        sequence: 0xFFFFFFFF,
        witness: Vec::new(),
        script_sig: Script::new(),
      });

      prev_txouts.insert(
        outpoint,
        TxOut {
          value: utxo.amount.as_sat(),
          script_pubkey: utxo.script_pub_key,
        },
      );
    }

    outputs.push(TxOut {
      script_pubkey: self.address.script_pubkey(),
      value: total_amount - spent_amount - (sat_per_byte * expected_size),
    });

    Ok((
      Transaction {
        input: inputs,
        output: outputs,
        lock_time: 0,
        version: 2,
      },
      prev_txouts,
    ))
  }

  pub async fn stats(&self) -> ConstataResult<Stats> {
    let balance = self
      .client
      .list_unspent(None, None, Some(&[&self.address]), None, None)?
      .iter()
      .map(|a| a.amount.as_sat())
      .sum();
    let info = self.client.get_blockchain_info()?;

    Ok(Stats {
      balance,
      address: self.address.clone(),
      block_height: info.blocks,
      block_hash: info.best_block_hash,
      current_bulletin: self.site.bulletin().current().await?,
    })
  }
}

describe! {
  use bitcoin::{
    Txid,
    util::psbt::serialize::Serialize,
  };
  use std::str::FromStr;

  test!{ creates_from_site_and_password
    let blockchain = Blockchain::from_site(Site::default_with_keyring("password").await?)
      .await.unwrap();

    assert_that!(
      &blockchain.address.to_string(),
      eq("bcrt1qsj2h8ernt4amc674l60vu925flvn57ff9lyry2".into())
    );
  }

  regtest!{ processes_sequentially (_db, c, mut chain)
    c.alice().await.stories_with_signed_docs(&b"first_batch"[..]).await;

    chain.fund_signer_wallet();

    assert!(chain.blockchain.process().await?.is_proposed());

    for _i in 0..5 {
      assert!(chain.blockchain.process().await?.is_submitted());
    }

    chain.mine_blocks(1);
    let submitted = chain.blockchain.process().await?.in_submitted()?;

    assert_that!(&submitted.raw_transaction(), rematch(&submitted.hash()));

    chain.mine_blocks(1);
    assert!(chain.blockchain.process().await?.is_draft());
  }

  regtest!{ does_not_propose_new_draft_until_some_time_passes (_site, c, _chain)
    use crate::models::{Site, SiteSettings};

    let settings = SiteSettings{
      minimum_bulletin_interval: 60,
      .. SiteSettings::default().unwrap()
    };

    let mut blockchain = Blockchain::from_site(
      Site::from_settings_and_password(settings, Some("password")).await?
    ).await.unwrap();

    // At least one document must be created for a bulletin to exist.
    c.alice().await.add_funds().await.signed_documents(b"").await;

    for _i in 0..5 {
      assert!(!blockchain.process().await?.is_proposed());
    }
  }

  regtest!{ builds_transaction_for_op_return (_db, _c, chain)
    chain.fund_signer_wallet();
    let utxos = chain.blockchain.get_utxos(Some(1)).unwrap();
    let fee = chain.blockchain.sats_per_byte_economy().unwrap();
    let result = chain.blockchain.build_transaction(&hex::decode(b"deadbeef")?, utxos, fee);

    assert_that!(&result, is_variant!(Ok));
    assert_that!(&hex::encode(result?.0.serialize()), eq("\
      020000000128a39da0c5eb71f25ede66e46175f3dd00e0321f96a8028f09f13446\
      1d40620f0000000000ffffffff022202000000000000066a04deadbeef9cd7052a\
      01000000160014849573e4735d7bbc6bd5fe9ece15544fd93a792900000000\
    ".to_string()))
  }

  regtest!{ fails_to_build_transaction_without_funds (_db, _c, chain)
    let utxos = chain.blockchain.get_utxos(Some(1)).unwrap();
    let fee = chain.blockchain.sats_per_byte_economy().unwrap();
    assert_that!(
      &chain.blockchain.build_transaction(&[0,1,0,1,0,1], utxos, fee),
      is_variant!(Err)
    );
  }

  regtest!{ can_submit_and_sync_a_proposal (site, c, mut chain)
    chain.fund_signer_wallet();
    let proposed = make_proposed_bulletin(&site, &c).await;

    let submitted = chain.blockchain.submit(proposed).await.unwrap();

    assert!(&chain.blockchain.sync_submitted(submitted.clone()).await?.is_none());
    chain.mine_blocks(2);
    assert!(&chain.blockchain.sync_submitted(submitted).await?.is_some());
  }

  regtest!{ automatically_resubmits_if_failed (site, c, mut chain)
    chain.regtest.change_relay_fee(Some("0.001"));
    chain.fund_signer_wallet();

    let proposed = make_proposed_bulletin(&site, &c).await;

    assert_that!(
      &chain.blockchain.submit(proposed.clone()).await.unwrap_err().to_string(),
      rematch("min relay fee not met")
    );

    chain.regtest.change_relay_fee(None);
    // Here we need to fund again because the block state is lost when changing relay fee.
    chain.fund_signer_wallet();

    let submitted = chain.blockchain.submit(proposed).await?;

    assert!(&chain.blockchain.sync_submitted(submitted.clone()).await?.is_none());
    chain.mine_blocks(3);
    assert!(&chain.blockchain.sync_submitted(submitted).await?.is_some());
  }

  regtest!{ can_speed_up_last_bulletin_transaction (site, c, mut chain)
    chain.fund_signer_wallet();
    let proposed = make_proposed_bulletin(&site, &c).await;

    // The bulletin tx fee is 6210 sats, paid directly from block reward.
    let previous_balance = get_balance(&chain).await?;
    let submitted = chain.blockchain.submit(proposed).await.unwrap();
    assert_number_of_bumps(&submitted, 0).await;
    let bulletin_fee = get_fees(&chain, previous_balance, &submitted.txid()).await?;
    assert_eq!(bulletin_fee , 6210);

    // The first bump fee is 6210 sats * 1.1.
    let first_fee = make_automated_bump(&mut chain, &submitted).await?;
    assert_number_of_bumps(&submitted, 1).await;
    assert_eq!(first_fee, (6210.0 * 1.1).to_u64().unwrap());

    assert_eq!(
      submitted.last_bump().await?.transaction().input[0].previous_output.txid,
      submitted.txid()
    );

    // The second bump fee is 6210 sats * 1.2.
    let second_fee = make_automated_bump(&mut chain, &submitted).await?;
    assert_number_of_bumps(&submitted, 2).await;
    assert_eq!(second_fee, (6210.0 * 1.2).to_u64().unwrap());

    // Testing limit of automated bumps (2)
    assert!(&chain.blockchain.sync_submitted(submitted.clone()).await?.is_none());
    assert_number_of_bumps(&submitted, 2).await;

    // The third bump fee is 6210 sats * 1.3.
    let third_fee = make_manual_bump(&mut chain, &submitted).await?;
    assert_number_of_bumps(&submitted, 3).await;
    assert_eq!(third_fee, (6210.0 * 1.3).to_u64().unwrap());

    // The fourth bump fee is 6210 sats * 1.4.
    let fourth_fee = make_manual_bump(&mut chain, &submitted).await?;
    assert_number_of_bumps(&submitted, 4).await;
    assert_eq!(fourth_fee, (6210.0 * 1.4).to_u64().unwrap());

    // Now the bulletin is submitted and synced, trying to bump the fee will fail.
    chain.mine_blocks(3);
    assert!(&chain.blockchain.sync_submitted(submitted).await?.is_some());
    assert_that!(
      &chain.blockchain.bump_fee().await.unwrap_err(),
      structure!{ Error::Stamping[ eq("no_current_submitted_bulletin".to_string()) ] }
    );
  }

  regtest!{ can_resubmit_regenerating_transaction (site, c, mut chain)
    chain.fund_signer_wallet();

    // By setting the default fee to 0 we create a problem for
    // propagating this transaction because of the min relay fee.
    // The transaction will always be invalid as it has 0 fee.
    chain = TestBlockchain{
      blockchain: Blockchain{ default_fee: 0, ..chain.blockchain },
      .. chain
    };

    let proposed = make_proposed_bulletin(&site, &c).await;

    assert_that!(
      &chain.blockchain.submit(proposed).await.unwrap_err().to_string(),
      rematch("min relay fee not met, 0")
    );

    let submitted = site.bulletin().current().await?.in_submitted()?;
    let old_txid = submitted.txid();

    // The fee issue is fixed, but the transaction remains invalid
    chain = TestBlockchain{ blockchain: Blockchain{ default_fee: 30_000, ..chain.blockchain }, .. chain };
    chain.mine_blocks(10);
    assert!(&chain.blockchain.sync_submitted(submitted.clone()).await?.is_none());

    // We fix the transaction and resubmit it.
    let (old_tx, tx, resubmitted) = chain.blockchain.resubmit().await?;

    assert_eq!(old_tx.txid(), old_txid);
    assert!(old_tx.txid() != tx.txid(), "Transactions must have changed");

    assert!(&chain.blockchain.sync_submitted(resubmitted.clone()).await?.is_none());
    chain.mine_blocks(3);
    assert!(&chain.blockchain.sync_submitted(resubmitted.clone()).await?.is_some());

    assert_that!(
      &chain.blockchain.resubmit().await.unwrap_err(),
      structure!{ Error::Stamping[ eq("no_current_submitted_bulletin".to_string()) ] }
    );
  }

  regtest!{ shows_blockchain_status (site, c, mut chain)
    chain.fund_signer_wallet();
    chain.blockchain.submit(make_proposed_bulletin(&site, &c).await).await?;

    let address = Address::from_str("bcrt1qsj2h8ernt4amc674l60vu925flvn57ff9lyry2")?;
    let stats = chain.blockchain.stats().await?;
    assert_eq!(stats.address, address);
    assert_eq!(stats.balance, 4999993244);
    assert_eq!(stats.block_height, 101);
  }

  regtest!{ it_populates_missing_block_times_on_old_bulletins (site, c, mut chain)
    c.alice().await.add_funds().await.signed_document(b"hello").await;
    chain.fund_signer_wallet();
    chain.simulate_stamping().await;
    let bulletin = site.bulletin().find(1).await?;
    let old_time = bulletin.attrs.block_time.clone();
    let new_bulletin = bulletin.update().block_time(None).save().await?;
    assert!(new_bulletin.block_time().is_none());
    site.bulletin().populate_block_times().await?;
    assert_eq!(new_bulletin.reloaded().await?.block_time(), &old_time);
  }

  async fn make_proposed_bulletin(site: &Site, c: &TestDb) -> Proposed {
    c.alice().await.stories_with_signed_docs(b"").await;
    site.bulletin().current().await.unwrap()
      .in_draft().unwrap()
      .propose().await.unwrap()
  }

  async fn assert_number_of_bumps(submitted: &Submitted, number_of_bump: i32) {
    assert_eq!(submitted.bump_count().await.unwrap(), number_of_bump);
    if number_of_bump != 0 {
      assert_eq!(submitted.last_bump().await.unwrap().counter(), &number_of_bump);
    }
  }

  async fn get_balance(chain: &TestBlockchain) -> ConstataResult<u64> {
    Ok(
      chain.blockchain.client
        .list_unspent(None, None, Some(&[&chain.blockchain.address]), None, None)?
        .iter().map(|i| i.amount.as_sat()).sum()
    )
  }

  async fn get_fees(chain: &TestBlockchain, previous_balance: u64, txid: &Txid) -> ConstataResult<u64> {
    let outputs_transaction_from_node: u64 = chain.blockchain.client
      .get_raw_transaction(txid, None)?.output.iter().map(|i| i.value).sum();
    Ok(previous_balance - outputs_transaction_from_node)
  }

  async fn make_automated_bump(chain: &mut TestBlockchain, submitted: &Submitted) -> ConstataResult<u64> {
    let previous_balance = get_balance(&chain).await?;
    assert!(&chain.blockchain.sync_submitted(submitted.clone()).await?.is_none());
    get_fees(chain, previous_balance, &submitted.last_bump().await?.txid()).await
  }
  async fn make_manual_bump(chain: &mut TestBlockchain, submitted: &Submitted) -> ConstataResult<u64> {
    let previous_balance = get_balance(&chain).await?;
    chain.blockchain.bump_fee().await.unwrap();
    get_fees(chain, previous_balance, &submitted.last_bump().await?.txid()).await
  }
}
