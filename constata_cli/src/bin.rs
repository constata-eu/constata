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
  AppendEntriesToIssuance(append_entries_to_issuance::Query),

  /// Gets your organization's account state
  AccountState(account_state::Query),

  /// Run a custom graphql query authenticated with your credentials.
  Graphql(graphql::Query),
}

fn main() {
  if let Err(e) = run() {
    println!("An error ocurred: {}", e);
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
