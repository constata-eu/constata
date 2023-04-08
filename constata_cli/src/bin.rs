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
  PreviewEntry(preview_entry::Query),

  /// Exports the verifiable HTML for an entry. Only available for entries in the 'done' state.
  EntryHtmlExport(entry_html_export::Query),

  /// Exports the unsigned ZIP file with the entry contents, useful for signing.
  /// If you want an approximation of how an entry will look look at the preview-entry command.
  UnsignedEntryPayload(unsigned_entry_payload::Query),

  /// Gets an HTML preview of some entry in the given issue. Use when you don't care about a specific entry.
  PreviewSampleFromIssuance(preview_sample_from_issuance::Query),

  /// Sign all entries in an issuance.
  /// This will download all entries and digitally sign them locally with your secure digital signature.
  SignIssuance(sign_issuance::Query),

  /// Export an issuance as a CSV file at any point.
  /// The exported issuance maintains the row ordering.
  IssuanceExport(issuance_export::Query),

  /// Lists all the templates you can use for your Issuances
  AllTemplates(all_templates::Query),

  /// Creates a new attestation of some files.
  CreateAttestation(create_attestation::Query),

  /// Downloads a verifiable HTML document from an attestation.
  AttestationHtmlExport(attestation_html_export::Query),

  /// Lists all your attestations
  AllAttestations(all_attestations::Query),

  /// Checks the state of an attestation, optionally waiting until it reaches that state.
  AttestationState(attestation_state::Query),

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
    Commands::PreviewEntry(query) => {
      let result = query.run(&client)?;
      if query.out_file.is_none() {
        printit(&result)?;
      } else {
        println!("Preview for {} saved to file", result.id);
      }
    },
    Commands::EntryHtmlExport(query) => {
      let result = query.run(&client)?;
      if query.out_file.is_none() {
        printit(&result)?;
      } else {
        println!("Verifiable HTML for Entry {} saved to file", result.id);
      }
    },
    Commands::UnsignedEntryPayload(query) => {
      let result = query.run(&client)?;
      if query.out_file.is_none() {
        printit(&result)?;
      } else {
        println!("ZIP file with raw contents of entry {} was saved", result.id);
      }
    },
    Commands::PreviewSampleFromIssuance(query) => {
      let has_out_file = query.out_file.is_none();
      let result = query.run(&client)?;

      if has_out_file {
        printit(&result)?;
      } else {
        println!("Preview for entry {} saved to file", result.id);
      }
    },
    Commands::IssuanceExport(query) => {
      let has_out_file = query.out_file.is_none();
      let result = query.run(&client)?;

      if has_out_file {
        printit(&result)?;
      } else {
        println!("Export for issuance {} saved to file", result.id);
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
    Commands::AttestationState(query) => {
      let value = query.run(&client)?;
      println!("{}", value);
      std::process::exit(if value { 0 } else { 1 })
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
    Commands::CreateAttestation(query) => {
      printit(&query.run(&client)?)?;
    },
    Commands::AttestationHtmlExport(query) => {
      let has_out_file = query.out_file.is_none();
      let result = query.run(&client)?;

      if has_out_file {
        printit(&result)?;
      } else {
        println!("Verifiable HTML for Attestation {} saved to file", result.id);
      }
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
