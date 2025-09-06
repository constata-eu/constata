pub mod selenium;
pub use selenium::Selenium;
pub use thirtyfour::{error::WebDriverResult, prelude::*, WebDriver, WebElement};

#[macro_export]
macro_rules! integration_test {
  ($i:ident($c:ident, $driver:ident) $($e:tt)* ) => {
    test!{ $i
      time_test::time_test!("integration test");
      let $c = TestDb::new().await?;
      let mut server = public_api_server::PublicApiServer::start();
      let $driver = Selenium::start().await;
      {$($e)*};
      server.stop();
      $driver.stop().await;
    }
  }
}

#[macro_export]
macro_rules! api_integration_test {
  ($i:ident($c:ident) $($e:tt)* ) => {
    test!{ $i
      time_test::time_test!("integration test");
      let $c = TestDb::new().await?;
      let mut server = public_api_server::PublicApiServer::start();
      //let $($chain)+ = TestBlockchain::new().await;
      {$($e)*};
      server.stop();
    }
  }
}

#[macro_export]
macro_rules! integration_test_private {
  ($i:ident($c:ident, $driver:ident) $($e:tt)* ) => {
    test!{ $i
      time_test::time_test!("integration test");
      let $c = TestDb::new().await?;
      let mut server = private_api_server::PrivateApiServer::start();
      let $driver = Selenium::start().await;
      {$($e)*};
      server.stop();
      $driver.stop().await;
    }
  }
}
