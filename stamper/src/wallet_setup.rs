use bitcoin::network::constants::Network;
use bitcoin::Address;
use bitcoin_wallet::{
  account::{Account, AccountAddressType, MasterAccount, MasterKeyEntropy, Unlocker},
  mnemonic::Mnemonic,
};
use constata_lib::Result;
use dialoguer::{theme::ColorfulTheme, Password, Select};

#[derive(Debug)]
#[allow(dead_code)]
pub struct WalletSetup {
  encrypted_hex: String,
  extended_pubkey: String,
  address: Address,
}

impl WalletSetup {
  pub fn new(network: Network, passphrase: &str) -> Result<Self> {
    let mnemonic = Mnemonic::new_random(MasterKeyEntropy::Paranoid)?;
    let mut master = MasterAccount::from_mnemonic(&mnemonic, 0, network, passphrase, None)?;

    let mut unlocker = Unlocker::new_for_master(&master, passphrase)?;

    master.add_account(Account::new(
      &mut unlocker,
      AccountAddressType::P2WPKH,
      1,
      0,
      10,
    )?);

    let address = master
      .get_mut((1, 0))
      .unwrap()
      .get_key(1)
      .unwrap()
      .address
      .clone();

    Ok(WalletSetup {
      address,
      encrypted_hex: hex::encode(master.encrypted()),
      extended_pubkey: master.master_public().to_string(),
    })
  }
}

fn main() {
  let password = Password::with_theme(&ColorfulTheme::default())
    .with_prompt("Wallet in-memory password")
    .with_confirmation("Repeat password", "Error: the passwords don't match.")
    .interact()
    .expect("Password prompt");

  println!("Your password is {} characters long", password.len());

  let selections = &[Network::Bitcoin, Network::Regtest, Network::Testnet];

  let network_id = Select::with_theme(&ColorfulTheme::default())
    .with_prompt("Pick your network")
    .default(0)
    .items(selections)
    .interact()
    .unwrap();

  println!(
    "{:#?}",
    WalletSetup::new(selections[network_id], &password).unwrap()
  )
}

constata_lib::describe_one! {
  use std::str::FromStr;
  use bitcoin::util::bip32::ExtendedPubKey;

  test!{ attempts_wallet_setup
    let password = "supersecret";
    let network = Network::Regtest;

    let result = WalletSetup::new(network, password).unwrap();

    let pubkey = ExtendedPubKey{ network, ..ExtendedPubKey::from_str(&result.extended_pubkey).unwrap()};
    let mut restored = MasterAccount::from_encrypted(
      &hex::decode(result.encrypted_hex).unwrap(),
      pubkey,
      0
    );
    let mut unlocker = Unlocker::new_for_master(&restored, password).unwrap();
    restored.add_account(
      Account::new(&mut unlocker, AccountAddressType::P2WPKH, 1, 0, 10).unwrap()
    );
    let address = restored.get_mut((1,0)).unwrap().get_key(1).unwrap().address.clone();

    assert_that!(&result.address, eq(address));
  }
}
