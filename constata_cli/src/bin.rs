use clap::{Parser, Subcommand};
use std::path::PathBuf;
use constata_client_lib::*;
use dialoguer::{theme::ColorfulTheme, Password};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
  /// Sets a custom config file
  #[arg(short, long, value_name = "FILE")]
  config: Option<PathBuf>,

  /// Use this daily password. Will prompt for a password if missing.
  #[arg(short, long, value_name = "FILE")]
  password: Option<String>,

  #[command(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  /// Start an issuance using a json array of (not nested) objects as initial entries.
  CreateIssuanceFromJson(public_api::controllers::certos::public_graphql::issuance_graphql::CreateIssuanceFromJsonInput),

  /// Start an issuance using a CSV file as initial entries.
  CreateIssuanceFromCsv(create_issuance_from_csv::Query),

  /// Append entries to a previously created issuance before signing it. 
  AppendEntriesToIssuance(public_api::controllers::certos::public_graphql::issuance_graphql::AppendEntriesToIssuanceInput),

  /// Lists your issuances
  AllIssuances(all_issuances::Query),

  /// Queries an issuance's state to check if it is created
  IsIssuanceCreated(is_issuance_created::Query),

  /// Queries an issuance's state to check if it is done
  IsIssuanceDone(is_issuance_done::Query),

  /// Lists entries across all Issuances
  AllEntries(all_entries::Query),

  /// Gets an HTML preview of a specific entry so you can have a look before signing
  Preview(preview::Query),

  /// Gets an HTML preview of some entry in the given issue. Use when you don't care about a specific entry.
  PreviewSampleFromIssuance(preview_sample_from_issuance::Query),

  /// Sign all entries in an issuance.
  /// This will download all entries and digitally sign them locally with your secure digital signature.
  SignIssuance(sign_issuance::Query),

  /// Lists all the templates you can use for your Issuances
  AllTemplates(all_templates::Query),

  /// Lists all your attestations
  AllAttestations(all_attestations::Query),
  /*

  constata-cli get-sample-entry-preview <issuance_id>

      or if you want to be specific:

      constata-cli all-entries --filter

      constata-cli issuance get-entry-preview <entry_id>

  then you can sign all entries:

  constata-cli sign-issuance sign <id>

  /* And there's more */

  constata-cli issuance list ...

  constata-cli entries list ...

  constata-cli issuance export <issuance_id>
  */

  /// Gets your organization's account state
  AccountState(account_state::Query),

  /// Run a custom graphql query authenticated with your credentials.
  Graphql(graphql::Query),
}

fn main() {
  if let Err(e) = run() {
    println!("An error ocurred: {}", e);
    std::process::exit(1);
  }
}

fn run() -> ClientResult<()> {
  let cli = Cli::parse();

  let daily_pass = cli.password
    .map(|i| i.to_string())
    .unwrap_or_else(|| {
      Password::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter your password")
        .interact()
        .unwrap()
    });

  let client = Client::from_config_file(cli.config, &daily_pass)?;

  match cli.command {
    Commands::CreateIssuanceFromJson(input) => {
      let query = create_issuance_from_json::Query{ input };
      printit(&query.run(&client)?)?;
    },
    Commands::CreateIssuanceFromCsv(query) => {
      printit(&query.run(&client)?)?;
    },
    Commands::AppendEntriesToIssuance(input) => {
      let query = append_entries_to_issuance::Query{ input };
      printit(&query.run(&client)?)?;
    },
    Commands::AllIssuances(query) => {
      printit(&query.run(&client)?)?;
    },
    Commands::AllEntries(query) => {
      printit(&query.run(&client)?)?;
    },
    Commands::Preview(query) => {
      let result = query.run(&client)?;
      if query.out_file.is_none() {
        printit(&result)?;
      } else {
        println!("Preview for {} saved to file", result.id);
      }
    },
    Commands::PreviewSampleFromIssuance(query) => {
      let has_out_file = query.out_file.is_none();
      let result = query.run(&client)?;

      if has_out_file {
        printit(&result)?;
      } else {
        println!("Preview for {} saved to file", result.id);
      }
    },
    Commands::SignIssuance(query) => {
      let entries = query.run(&client)?;
      println!("All {} entries signed", entries);
    },
    Commands::AllTemplates(query) => {
      printit(&query.run(&client)?)?;
    },
    Commands::AllAttestations(query) => {
      printit(&query.run(&client)?)?;
    },
    Commands::IsIssuanceCreated(query) => {
      let value = query.run(&client)?;
      println!("{}", value);
      std::process::exit(if value { 0 } else { 1 })
    },
    Commands::IsIssuanceDone(query) => {
      let value = query.run(&client)?;
      println!("{}", value);
      std::process::exit(if value { 0 } else { 1 })
    },
    Commands::AccountState(query) => {
      printit(&query.run(&client)?)?;
    },
    Commands::Graphql(query) => {
      printit(&query.run(&client)?)?;
    },
  }

  Ok(())
}

fn printit<T: serde::Serialize>(it: &T) -> ClientResult<()>{
  println!("{}", serde_json::to_string_pretty(it)?);
  Ok(())
}
