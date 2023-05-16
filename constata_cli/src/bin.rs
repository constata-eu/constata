mod runner;
use runner::*;

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
  ///
  /// The id of your first issuance when calling all-issuances should be --json-pointer=/entries/0/id
  ///
  /// Exits with an error if the pointer is invalid or not found.
  ///
  /// See https://www.rfc-editor.org/rfc/rfc6901 for pointer syntax.
  #[arg(short, long)]
  json_pointer: Option<String>,

  #[command(subcommand)]
  command: Commands,
}

commands! {
  /// Start an issuance using a json array of objects as initial entries
  ///
  /// Each entry should be a flat object with strings as values, like:
  /// { 
  ///   "motive":"Big Data Analyst",
  ///   "place":"Madrid, Spain",
  ///   "date":"March 3, 2023",
  ///   "recipient_identification":"Johnny777",
  ///   "name":"John Doe",
  ///   "shared_text":"Thank you for participating",
  ///   "email":"john@example.com",
  ///   "custom_text":"Large custom text"
  /// }
  CreateIssuanceFromJson => print_json,

  /// Start an issuance using a CSV file as initial entries.
  CreateIssuanceFromCsv => print_json,

  /// Append entries to a previously created issuance before signing it. 
  AppendEntriesToIssuance => print_json,

  /// Lists your issuances
  AllIssuances => print_json,

  /// Queries an issuance's state, optionally waiting until the expected state is reached.
  IssuanceState => |runner, query| {
    runner.exit_with_boolean(query.run(&runner.client)?)?;
  },

  /// Lists entries across all Issuances
  AllEntries => print_json,
  
  /// Gets an HTML preview of a specific entry so you can have a look before signing
  PreviewEntry => print_json_or_save("Preview for entry #{} saved to file"),

  /// Exports the verifiable HTML for an entry.
  ///
  /// Only available for entries in the 'done' state.
  /// See the entry-state to check for the entry status before calling.
  EntryHtmlExport => print_json_or_save("Verifiable HTML for Entry {} saved to file"),

  /// Exports all verifiable HTMLs from entries matching the given criteria
  ///
  /// Use all-entries to review your query before downloading.
  AllEntriesHtmlExport => |runner, query| {
    use std::io::Write;
    let result = query.run(&runner.client, |current, total, entry| {
      print!("\rProcessing entry {:>5}, its {:>5}/{:<5}", entry.id, current, total);
      let _ = std::io::stdout().flush();
    })?;
    println!("\nSaved {} verifiable HTMLs", result);
  },

  /// Exports the unsigned ZIP file with the entry contents, useful for signing.
  /// 
  /// If you want an approximation of how an entry will look look at the preview-entry command.
  UnsignedEntryPayload => print_json_or_save("ZIP file with raw contents for entry {} was saved"),

  /// Gets an HTML preview of some entry in the given issue. Use when you don't care about a specific entry.
  PreviewSampleFromIssuance => print_json_or_save("Preview for entry {} saved to file"),

  /// Sign all entries in an issuance.
  ///
  /// This will download all entries and digitally sign them locally with your secure digital signature.
  SignIssuance => |runner, query| {
    use std::io::Write;
    query.run(&runner.client, |i| {
      print!("\rSigning entry {:>5} of {:>5}", i.current, i.total);
      let _ = std::io::stdout().flush();
    })?;
  },

  /// Export an issuance as a CSV file at any point.
  ///
  /// The exported issuance maintains the row ordering, and adds useful columns for each entry.
  /// You can use this export file to load up on another system, like mailchimp for campaigns.
  IssuanceExport => print_json_or_save("Export for Issuance #{} saved to file"),

  /// Lists all the templates you can use for your Issuances
  AllTemplates => print_json,

  /// Creates a new attestation of some files.
  CreateAttestation => print_json,

  /// Lists all your attestations
  AllAttestations => print_json,

  /// Downloads a verifiable HTML document from an attestation.
  AttestationHtmlExport => print_json_or_save("Verifiable HTML for Attestation {} saved to file"),

  /// Exports all verifiable HTMLs from attestations matching the given criteria
  ///
  /// Use all-attestations to review your query before downloading.
  AllAttestationsHtmlExport => |runner, query| {
    use std::io::Write;
    let result = query.run(&runner.client, |current, total, attestation| {
      print!("\rProcessing attestation {:>5}, its {:>5}/{:<5}", attestation.id, current, total);
      let _ = std::io::stdout().flush();
    })?;
    println!("\nSaved {} verifiable HTMLs", result);
  },

  /// Checks the state of an attestation, optionally waiting until it reaches that state.
  AttestationState => |runner, query| { 
    runner.exit_with_boolean(query.run(&runner.client)?)?;
  },

  /// Publish/unpublish an attestation so people can see it on constata's website.
  AttestationSetPublished => print_json,

  /// Gets your organization's account state
  AccountState => print_json,

  /// Sets your web callbacks URL
  ///
  /// We will send a web callback to that URL when your attestations change state.
  /// Call with an empty URL to reset it.
  UpdateWebCallbacksUrl => print_json,

  /// Lists your web callbacks, for debugging and recovery.
  ///
  /// If you think you've missed any web callback, you can check here.
  /// The whole web callback body is included so you can reprocess it manually.
  AllWebCallbacks => print_json,

  /// Validate Web Callback, and outputs its contents if valid.
  ValidateWebCallback => print_json,

  /// Run a custom graphql query authenticated with your credentials.
  ///
  /// Gain access to new API features without a new client.
  /// Improve data transfer size by sending custom optimized queries.
  CustomGraphql => print_json,
}

impl Cli {
  fn run(self) -> ClientResult<()> {
    let daily_pass = self.password
      .map(|i| i.to_string())
      .unwrap_or_else(|| {
        Password::with_theme(&ColorfulTheme::default())
          .with_prompt("Enter your password")
          .interact()
          .unwrap()
      });

    Runner::run(
      Client::from_config_file(self.config, &daily_pass)?,
      self.json_pointer,
      self.command
    )
  }
}

fn main() {
  if let Err(e) = Cli::parse().run() {
    println!("An error ocurred: {}", e);
    std::process::exit(1);
  }
}
