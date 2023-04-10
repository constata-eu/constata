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

  /// Extract a specific attribute from a successful JSON response.
  /// The id of your first issuance when calling all-issuances should be --json-pointer=/entries/0/id
  /// Exits with an error if the pointer is invalid or not found.
  /// See https://www.rfc-editor.org/rfc/rfc6901 for pointer syntax.
  #[arg(short, long)]
  json_pointer: Option<String>,

  #[command(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  /// Start an issuance using a json array of (not nested) objects as initial entries.
  CreateIssuanceFromJson(create_issuance_from_json::Query),

  /// Start an issuance using a CSV file as initial entries.
  CreateIssuanceFromCsv(create_issuance_from_csv::Query),

  /// Append entries to a previously created issuance before signing it. 
  AppendEntriesToIssuance(append_entries_to_issuance::Query),

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

  /// Exports all verifiable HTMLs from entries matching the given criteria
  AllEntriesHtmlExport(all_entries_html_export::Query),

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

  /// Lists all your attestations
  AllAttestations(all_attestations::Query),

  /// Downloads a verifiable HTML document from an attestation.
  AttestationHtmlExport(attestation_html_export::Query),

  /// Exports all verifiable HTMLs from attestations matching the given criteria
  AllAttestationsHtmlExport(all_attestations_html_export::Query),

  /// Checks the state of an attestation, optionally waiting until it reaches that state.
  AttestationState(attestation_state::Query),

  /// Gets your organization's account state
  AccountState(account_state::Query),

  /// Run a custom graphql query authenticated with your credentials.
  /// Gain access to new API features without a new client.
  /// Improve data transfer size by sending custom optimized queries.
  Graphql(graphql::Query),
}

impl Cli {
  fn run(&self) -> ClientResult<()> {
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
      Commands::CreateIssuanceFromJson(query) => {
        self.print_json(&query.run(&client)?)?;
      },
      Commands::CreateIssuanceFromCsv(query) => {
        self.print_json(&query.run(&client)?)?;
      },
      Commands::AppendEntriesToIssuance(query) => {
        self.print_json(&query.run(&client)?)?;
      },
      Commands::AllIssuances(query) => {
        self.print_json(&query.run(&client)?)?;
      },
      Commands::AllEntries(query) => {
        self.print_json(&query.run(&client)?)?;
      },
      Commands::AllEntriesHtmlExport(query) => {
        use std::io::Write;
        let result = query.run(&client, |current, total, entry| {
          print!("\rProcessing entry {:>5}, its {:>5}/{:<5}", entry.id, current, total);
          let _ = std::io::stdout().flush();
        })?;
        println!("\nSaved {} verifiable HTMLs", result);
      },
      Commands::AllAttestationsHtmlExport(query) => {
        use std::io::Write;
        let result = query.run(&client, |current, total, entry| {
          print!("\rProcessing attestation {:>5}, its {:>5}/{:<5}", entry.id, current, total);
          let _ = std::io::stdout().flush();
        })?;
        println!("\nSaved {} verifiable HTMLs", result);
      },
      Commands::PreviewEntry(query) => {
        let result = query.run(&client)?;
        if query.out_file.is_none() {
          self.print_json(&result)?;
        } else {
          println!("Preview for {} saved to file", result.id);
        }
      },
      Commands::EntryHtmlExport(query) => {
        let result = query.run(&client)?;
        if query.out_file.is_none() {
          self.print_json(&result)?;
        } else {
          println!("Verifiable HTML for Entry {} saved to file", result.id);
        }
      },
      Commands::UnsignedEntryPayload(query) => {
        let result = query.run(&client)?;
        if query.out_file.is_none() {
          self.print_json(&result)?;
        } else {
          println!("ZIP file with raw contents of entry {} was saved", result.id);
        }
      },
      Commands::PreviewSampleFromIssuance(query) => {
        let has_out_file = query.out_file.is_none();
        let result = query.run(&client)?;

        if has_out_file {
          self.print_json(&result)?;
        } else {
          println!("Preview for entry {} saved to file", result.id);
        }
      },
      Commands::IssuanceExport(query) => {
        let has_out_file = query.out_file.is_none();
        let result = query.run(&client)?;

        if has_out_file {
          self.print_json(&result)?;
        } else {
          println!("Export for issuance {} saved to file", result.id);
        }
      },
      Commands::SignIssuance(query) => {
        use std::io::Write;
        query.run(&client, |i: &sign_issuance::Iter| {
          print!("\rSigning entry {:>5} of {:>5}", i.current, i.total);
          let _ = std::io::stdout().flush();
        })?;
      },
      Commands::AllTemplates(query) => {
        self.print_json(&query.run(&client)?)?;
      },
      Commands::AllAttestations(query) => {
        self.print_json(&query.run(&client)?)?;
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
        self.print_json(&query.run(&client)?)?;
      },
      Commands::AttestationHtmlExport(query) => {
        let has_out_file = query.out_file.is_none();
        let result = query.run(&client)?;

        if has_out_file {
          self.print_json(&result)?;
        } else {
          println!("Verifiable HTML for Attestation {} saved to file", result.id);
        }
      },
      Commands::AccountState(query) => {
        self.print_json(&query.run(&client)?)?;
      },
      Commands::Graphql(query) => {
        self.print_json(&query.run(&client)?)?;
      },
    }

    Ok(())
  }

  fn print_json<T: serde::Serialize>(&self, it: &T) -> ClientResult<()>{
    let value = serde_json::to_value(&it)?;
    let as_str = serde_json::to_string(&it)?;

    let json = if let Some(pointer) = &self.json_pointer {
      value.pointer(&pointer).ok_or_else(||{
        error![InvalidInput("Could not find pointer {} on response {}", &pointer, as_str)]
      })?
    } else {
      &value
    };
    
    println!("{}", serde_json::to_string_pretty(json)?);
    Ok(())
  }
}

fn main() {
  if let Err(e) = Cli::parse().run() {
    println!("An error ocurred: {}", e);
    std::process::exit(1);
  }
}
