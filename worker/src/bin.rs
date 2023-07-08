use constata_lib::prelude::*;
use email_bot::EmailBot;
use log::*;
use std::time::Duration;
use tokio::sync::RwLock;
use std::collections::HashSet;
use std::sync::Arc;

use constata_lib::models::*;

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
      //println!("Running: {}", $name);
      if let Err(err) = { $($blk)* } {
        error!("Error in {}: {:?}", $name, err);
      }
    )
  }

  let prompts_site = site.clone();
  handles.push(tokio::spawn(async move{
    let lock: Arc<RwLock<HashSet<i32>>> = Arc::new(RwLock::new(HashSet::new()));

    loop {
      println!("Looping through pending requests");
      let started = lock.read().await.iter().cloned().collect::<Vec<i32>>();
      println!("Already started {:?}", &started);
      let pending = prompts_site
        .vc_request()
        .select()
        .id_not_in(started)
        .state_eq(VcRequestState::Pending)
        .all().await
        .unwrap().into_iter();
      for r in pending {
        let id = r.attrs.id;
        let mut n = lock.write().await;
        n.insert(id);

        let inner_lock = Arc::clone(&lock);
        tokio::spawn(async move {
          println!("Starting websocket for {}", id);
          match r.request_on_vidchain().await {
            Err(e) => println!("Error processing vc_request {}: {} ", id, e),
            Ok(_) => println!("Processed vc_request {}", id),
          }
          let mut n = inner_lock.write().await;
          n.remove(&id);
        });
      }
      tokio::time::sleep(Duration::from_millis(500)).await;
    }
  }));

  every![100, |s| {
    run!("workroom_create_received" { s.issuance().create_all_received().await });
    run!("workroom_complete_all_notified" { s.issuance().try_complete().await });
    run!("attempting_webhooks" { s.web_callback().attempt_all_pending().await });
  }];

  every![10000, |s| {
    match EmailBot::new(s.clone()).await {
      Ok(email_bot) => { run!("notify_emails" { email_bot.handle_notify_emails().await }); },
      Err(err) => error!("Error connecting to email bot: {:?}", err),
    };
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
