use constata_lib::models::{Site, Utc};
use email_bot::EmailBot;
use telegram_bot::TelegramBot;
use log::*;
use std::time::Duration;

#[tokio::main]
async fn main() {
  let site = Site::from_stdin_password().await.unwrap();
  site.audit_log.start();
  let mut handles = vec![];

  macro_rules! every {
    ($wait:expr, |$site:ident| {$($blk:tt)*}) => (
      let $site = site.clone();
      handles.push(tokio::spawn(async move {
        loop {
          { $($blk)* }
          tokio::time::sleep(Duration::from_millis($wait)).await;
        }
      }));
    )
  }

  macro_rules! run {
    ($name:literal {$($blk:tt)*}) => (
      println!("Running: {}", $name);
      if let Err(err) = { $($blk)* } {
        error!("Error in {}: {:?}", $name, err);
      }
    )
  }

  every![100, |s| {
    run!("workroom_create_received" { s.request().create_all_received().await });
    run!("workroom_complete_all_notified" { s.request().try_complete().await });
  }];

  every![10000, |s| {
    match EmailBot::new(s.clone()).await {
      Ok(email_bot) => {
        run!("witness_emails" { email_bot.witness_emails(50).await });
        run!("notify_emails" { email_bot.handle_notify_emails().await });
      },
      Err(err) => error!("Error connecting to email bot: {:?}", err),
    };
  }];

  every![2000, |s| {
    let bot = TelegramBot::new(s.clone());
    run!("telegram_bot_sync_updates" { bot.sync_updates().await });
    run!("telegram_bot_process_updates" { bot.process_updates().await });
    run!("telegram_bot_greet_group_chats" { bot.greet_group_chats().await });
    run!("telegram_bot_remind_group_chats" { bot.remind_group_chats().await });
    run!("telegram_bot_notify_private_chat_documents" { bot.notify_private_chat_documents().await });
    run!("telegram_bot_flush_outgoing_messages" { bot.flush_outgoing_messages().await });
  }];

  every![300000, |s| {
    run!("pubkey_domain_endorsement" { s.pubkey_domain_endorsement().process_all().await });
  }];

  every![300000, |s| {
    run!("delete_old_parked" { s.document().delete_old_parked().await });
    run!("parked_reminder_create_new_campaign" { s.parked_reminder().create_new_reminders(Utc::now()).await });
    run!("expire_old_invoices" { s.invoice().expire_all_old_invoices().await });
    run!("expire_old_access_tokens" { s.access_token().expire_all_old_access_tokens().await });
  }];

  futures::future::join_all(handles).await;
}
