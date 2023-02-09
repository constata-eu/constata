use constata_lib::error::*;
use constata_lib::models::{Site, admin_user::AdminRole};
use clap::{command, Command};
use dialoguer::{theme::ColorfulTheme, Password, Input};

#[tokio::main]
async fn main() {
  let matches = command!()
    .propagate_version(true)
    .subcommand_required(true)
    .arg_required_else_help(true)
    .subcommand(Command::new("create-admin").about("creates an admin user"))
    .subcommand(Command::new("populate-block-times").about("Populates bulletin block times"))
    .subcommand(Command::new("populate-backup-storage").about("Copies all files from current DO storage to AWS backup storage"))
    .subcommand(Command::new("check-template-schemas").about("Typechecks all stored template schemas"))
    .get_matches();

  match matches.subcommand() {
    Some(("create-admin", _)) => create_superadmin().await.unwrap(),
    Some(("populate-block-times", _)) => populate_block_times().await.unwrap(),
    Some(("populate-backup-storage", _)) => populate_backup_storage().await.unwrap(),
    Some(("check-template-schemas", _)) => check_template_schemas().await.unwrap(),
    _ => ()
  }
}

async fn populate_block_times() -> Result<()> {
  Site::default().await.expect("Cannot load site")
    .bulletin()
    .populate_block_times().await?;
  Ok(())
}

async fn create_superadmin() -> Result<()> {
  let site = Site::default().await.expect("Cannot load site");
  let username: String = Input::new()
    .with_prompt("New username")
    .interact_text()
    .expect("Error in username prompt");

  let password = Password::with_theme(&ColorfulTheme::default())
    .with_prompt("New password")
    .with_confirmation("Repeat password", "Error: the passwords don't match.")
    .interact()
    .expect("Error in password prompt");

  let otp_seed = site.admin_user()
    .create(&username, &password, AdminRole::SuperAdmin)
    .await
    .expect("Error creating admin user")
    .attrs
    .otp_seed;

  println!("User {username} created. QR for OTP will follow");
  println!("OTP seed is: {otp_seed}");
  qr2term::print_qr(&format!("otpauth://totp/{username}?secret={otp_seed}&issuer=constata.admin"))
    .expect("Could not print OTP seed");
  Ok(())
}

async fn populate_backup_storage() -> Result<()> {
  use constata_lib::models::Storable;
  let password = Password::with_theme(&ColorfulTheme::default())
    .with_prompt("Keyring password")
    .interact()
    .expect("Error in password prompt");

  let site = Site::default_with_keyring(&password).await.expect("Cannot load site");
  
  macro_rules! make_backup_copy {
    ($model:ident) => (
      {
        let count = site.$model().select().count().await?;
        println!("Processing {}:", stringify!($model));
        for (i, o) in site.$model().select().all().await?.iter().enumerate() {
          println!("{:04}/{:04}: {}\r", i, count, o.id());
          if o.storage_backup_fetch().await.is_ok() {
            println!("Skipping, already copied");
            continue;
          }
          match o.storage_fetch().await {
            Ok(bytes) => {
              o.storage_backup_put(&bytes).await?;
              assert_eq!(&o.storage_backup_fetch().await?, &bytes);
            },
            Err(e) => {
              println!("Warning: could not find {} in old storage. {:?}", o.storage_id(), e);
            }
          }
        }
        println!("Done with {}", stringify!($model));
      }
    )
  }

  make_backup_copy!(document_part);
  make_backup_copy!(pubkey_domain_endorsement);
  make_backup_copy!(template);
  make_backup_copy!(request);
  make_backup_copy!(entry);
  make_backup_copy!(kyc_request_evidence);
  make_backup_copy!(telegram_bot_update);

  Ok(())
}

async fn check_template_schemas() -> Result<()> {
  let password = Password::with_theme(&ColorfulTheme::default())
    .with_prompt("Keyring password")
    .interact()
    .expect("Error in password prompt");

  let site = Site::default_with_keyring(&password).await.expect("Cannot load site");

  for template in site.template().select().all().await? {
    print!(".");
    if let Err(e) = template.parsed_schema() {
      println!("Template {} has invalid schema {:?}", template.id(), e);
    }
  }
  Ok(())
}
