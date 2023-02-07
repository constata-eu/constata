use crate::models::{Blockchain, Site};
use bitcoincore_rpc::{bitcoin::Address, Auth, Client, RpcApi};
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use bitcoin::PrivateKey;

pub struct TestBlockchain {
  pub blockchain: Blockchain,
  pub regtest: Regtest,
}

impl TestBlockchain {
  pub async fn new() -> Self {
    Self {
      blockchain: Blockchain::from_site(Site::default_with_keyring("password").await.unwrap()).await.unwrap(),
      regtest: Regtest::new(None)
    }
  }

  pub async fn default_private_key() -> Result<PrivateKey, anyhow::Error> {
    Ok(Site::default_with_keyring("password").await?.keyring().unwrap().unwrap().private_key)
  }

  pub fn fund_signer_wallet(&self) {
    self.regtest.generate_confirmed(&self.blockchain.address);
  }

  pub fn mine_blocks(&self, count: u64) {
    self.regtest.generate(count);
  }

  pub async fn simulate_stamping(&mut self) {
    for _i in 0..5 {
      let _dontcare = self.blockchain.process().await;
      self.mine_blocks(2);
    }
  }
}

pub struct Regtest {
  child: Child,
  client: Client,
}

impl Regtest {
  pub fn new(min_relay_fee: Option<&str>) -> Self {
    let _ = std::fs::remove_dir_all("/tmp/regtest_node");
    Command::new("cp")
      .args(&[
        "-r",
        "../constata_lib/src/test_support/resources/regtest_node_base",
        "/tmp/regtest_node",
      ])
      .spawn()
      .unwrap();

    let child = Self::start_bitcoind(min_relay_fee);

    let client = Client::new(
      "http://localhost:10203".to_string(),
      Auth::UserPass("user".to_string(), "password".to_string()),
    )
    .expect("RPC unreachable");

    Regtest { child, client }
  }

  fn start_bitcoind(min_relay_fee: Option<&str>) -> Child {
    let mut child = Command::new("bitcoind")
      .args(&[
        "-datadir=/tmp/regtest_node",
        "-regtest",
        "-noconnect",
        "-txindex",
        "-rpcuser=user",
        "-rpcpassword=password",
        "-rpcport=10203",
        "-wallet=main",
        &format!("-minrelaytxfee={}", min_relay_fee.unwrap_or("0.00001")),
      ])
      .stdout(Stdio::piped())
      .spawn()
      .unwrap();

    for line in BufReader::new(child.stdout.take().unwrap()).lines() {
      if line.unwrap().contains("net thread start") {
        break;
      }
    }

    child
  }

  pub fn change_relay_fee(&mut self, min_fee: Option<&str>) {
    // Warning:
    // This is fast, but does not wait for bitcoind graceful exit
    // which may leave block state and transaccions unsaved.
    // So, in the middle of the spec that uses this, we end up
    // having to regenerate some blocks.
    self.child.kill().unwrap();
    self.child = Self::start_bitcoind(min_fee);
  }

  fn kill(&mut self) {
    self.child.kill().unwrap();
    let _ = std::fs::remove_dir_all("/tmp/regtest_node");
  }

  pub fn generate_to_address(&self, blocks: u64, address: &Address) {
    let _ = self.client.import_address(address, None, None);
    let _ = self.client.generate_to_address(blocks, address);
  }

  pub fn generate_confirmed(&self, address: &Address) {
    self.generate_to_address(101, address);
  }

  pub fn generate(&self, blocks: u64) {
    let address = self.client.get_new_address(None, None).unwrap();
    self.client.generate_to_address(blocks, &address).unwrap();
  }
}

impl std::ops::Drop for Regtest {
  fn drop(&mut self) {
    self.kill();
  }
}
