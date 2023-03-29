pub mod matchers;
pub mod public_api_server;
pub mod private_api_server;
pub mod samples;
pub mod test_blockchain;
pub mod test_db;
pub mod worker_server;
use std::future::Future;

pub use galvanic_assert::{
  self,
  matchers::{collection::*, variant::*, *},
  *,
};
pub use matchers::*;
pub use rocket::http::Status;
pub use test_blockchain::*;
pub use test_db::*;
pub use crate::models::site::SiteSettings;

#[macro_export]
macro_rules! test {
  ($i:ident $($e:tt)* ) => {

    #[test]
    fn $i() {

      async fn run_test() -> std::result::Result<(), anyhow::Error> {
        {$($e)*}
        Ok(())
      }

      let result = tokio::runtime::Runtime::new()
        .expect("could not build runtime")
        .block_on(run_test());

      if let Err(e) = result {
        let source = e.source().map(|e| e.to_string() ).unwrap_or_else(|| "none".to_string());
        println!("Error: {e:?}\n Source: {source}.");
        panic!("Error in test. see backtrace");
      }
    }
  }
}

#[macro_export]
macro_rules! requires_setting {
  ($( $i:ident ).+) => {
    let mut chars: Vec<char> = SiteSettings::default()?.$($i).+.chars().collect();
    chars.dedup();
    if chars.len() <= 4 {
      return Ok(())
    }
  }
}

pub fn wait_here() {
  use std::{thread, time};
  println!("Waiting here as instructed. ctrl+c to quit.");
  let ten_millis = time::Duration::from_millis(10);
  loop {
    thread::sleep(ten_millis);
  }
}

pub async fn try_until<T: Future<Output = bool>>(times: i32, err: &str,  call: impl Fn() -> T) {
  use std::{thread, time};
  let millis = time::Duration::from_millis(100);
  for _i in 0..times {
    if call().await {
      return;
    }
    thread::sleep(millis);
  }
  assert!(false, "{err}");
}

pub fn read(file: &str) -> Vec<u8> {
  std::fs::read(&format!("../constata_lib/src/test_support/resources/{file}")).unwrap()
}

pub fn read_to_string(file: &str) -> String {
  std::fs::read_to_string(&format!("../constata_lib/src/test_support/resources/{file}")).unwrap()
}

pub fn mock_callbacks_url(hits: usize, status: usize) -> mockito::Mock {
  mockito::mock("POST", "/callbacks_url")
    .with_status(status)
    .with_body("got it")
    .expect(hits)
    .create()
}

pub async fn assert_bulletin_payload(bulletin: &crate::models::Bulletin, count: usize, expected: Vec<&str>) {
  let payload = bulletin.payload().await.unwrap();
  let found: Vec<&str> = payload.trim_end_matches('\n').split('\n').collect();
  assert_that!(&found, contains_subset(expected));
  assert_that!(&found, sorted_ascending());
  assert_eq!(found.len(), count, "Found element count does not match expected count");
}

pub async fn wait_here_and<F, R, Fut>(millis: u64, f: F)
where
  F: Fn() -> Fut,
  Fut: Future<Output = R>,
  R: Sized + std::any::Any
{
  use std::{thread, time};
  println!("Waiting here and processing as instructed. ctrl+c to quit.");
  let ten_millis = time::Duration::from_millis(millis);
  loop {
    f().await;
    thread::sleep(ten_millis);
  }
}

#[macro_export]
macro_rules! dbtest {
  ($i:ident($($site:ident)+, $($c:ident)+) $($e:tt)* ) => {
    test!{ $i
      time_test::time_test!("db env");
      let c = TestDb::new().await?;
      let $($site)+ = c.site.clone();
      let $($c)+ = c;
      $($e)*
    }
  }
}

#[macro_export]
macro_rules! regtest {
  ($i:ident($($site:ident)+, $c:ident, $($chain:ident)+) $($e:tt)* ) => {
    test!{ $i
      time_test::time_test!("regtest env");
      let $c = TestDb::new().await?;
      let $($site)+ = $c.site.clone();
      let $($chain)+ = TestBlockchain::new().await;
      $($e)*
    }
  }
}

#[macro_export]
macro_rules! assert_document_part {
  ($item:expr,
   $signature_count:expr,
   $is_base:expr,
   $document_id:expr,
   $friendly_name:expr,
   $hash:expr,
   $content_type:expr,
   $size_in_bytes:expr
  ) => (
    assert_that!(&$item.attrs, structure![DocumentPartAttrs {
      id: rematch("[a-f0-9]{64}"),
      is_base: eq($is_base),
      document_id: rematch(&format!("{}-[a-f0-9]{{16}}", $document_id)),
      friendly_name: rematch($friendly_name),
      hash: rematch($hash),
      content_type: rematch($content_type),
      size_in_bytes: eq($size_in_bytes),
    }]);
    assert_eq!($item.document_part_signature_vec().await?.len(), $signature_count);
  )
}

#[macro_export]
macro_rules! assert_document_part_signature {
  ($item:expr,
   $id:expr,
   $document_part_id:expr,
   $pubkey_id:expr,
   $signature:expr,
   $signature_hash:expr,
   $bulletin_id:expr
  ) => (
    assert_that!($item, structure![DocumentPartSignatureAttrs {
      id: eq($id),
      document_part_id: rematch($document_part_id),
      pubkey_id: rematch($pubkey_id),
      signature: eq(hex::decode($signature).unwrap()),
      signature_hash: rematch($signature_hash),
      bulletin_id: eq($bulletin_id),
    }]);
  )
}
