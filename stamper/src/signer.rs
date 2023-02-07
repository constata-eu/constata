use clap::{crate_authors, crate_name, crate_version, App, Arg};
use constata_lib::bitcoin::util::psbt::serialize::Serialize;
use constata_lib::models::{Blockchain, Site};
use dialoguer::{theme::ColorfulTheme, Confirm};

#[tokio::main]
async fn main() {
  let matches = App::new(crate_name!())
    .version(crate_version!())
    .author(crate_authors!())
    .about("Signer service for constata.eu stamping")
    .arg(
      Arg::with_name("ACTION")
        .help("What mode to run the program in")
        .index(1)
        .possible_values(&["run", "bump", "resubmit"])
        .required(true),
    )
    .get_matches();

  let mut blockchain = Blockchain::from_site(Site::from_stdin_password().await.unwrap()).await.unwrap();

  match matches.value_of("ACTION").unwrap() {
    "run" => {
      let mut old = blockchain.site.bulletin().current().await.expect("fetching current bulletin");
      loop {
        if let Err(e) = blockchain.process().await {
          println!("Error processing {:?}", e);
          println!("Status: {:?}", blockchain.stats().await);
          std::thread::sleep(std::time::Duration::new(120, 0));
        }
        if let Ok(new) = blockchain.site.bulletin().current().await {
          if new != old {
            println!("Status: {:?}", blockchain.stats().await);
            old = new
          }
        }
      }
    }
    "bump" => match blockchain.bump_fee().await {
      Ok((tx, submitted)) => {
        println!("Bumping bulletin {:?}", submitted);
        println!("New transaction is {}", hex::encode(&tx.serialize()));
      }
      error => println!("Failed to bump transaction {:?}", error),
    },
    "resubmit" => {
      println!(
        "Don't do this unless you're sure the current bulletin transaction has not been propagated"
      );
      println!(
        "You risk having a bulletin fingerprint timestamped twice, which could be very confusing."
      );
      let confirm = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Are yous srsly going to do this?")
        .interact()
        .unwrap();

      if confirm {
        match blockchain.resubmit().await {
          Ok((old, new, submitted)) => {
            println!("Resubmitting bulletin {:?}", submitted);
            println!(
              "Old transaction was {}\n{}",
              old.txid(),
              hex::encode(&old.serialize())
            );
            println!(
              "New transaction is {}\n{}",
              new.txid(),
              hex::encode(&new.serialize())
            );
          }
          error => println!("Failed resubmitting transaction {:?}", error),
        }
      } else {
        println!("Wise choice");
      }
    }
    e => println!("Unknown option {}", e),
  }
}
