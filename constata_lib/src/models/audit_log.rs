use log::*;

use std::fs::File;

#[derive(Debug, Clone)]
pub struct AuditLog {
  file_name: String,
  max_size: u64,
}

impl AuditLog {
  pub fn new(file_name: String, max_size: u64) -> Self {
    Self {
      file_name,
      max_size,
    }
  }

  pub fn start(&self) {
    use log4rs::append::rolling_file::{
      policy::compound::{
        roll::fixed_window::FixedWindowRollerBuilder, trigger::size::SizeTrigger, CompoundPolicy,
      },
      RollingFileAppender,
    };
    use log4rs::config::{Appender, Config, Logger, Root};
    use log4rs::encode::json::JsonEncoder;

    let policy = CompoundPolicy::new(
      Box::new(SizeTrigger::new(self.max_size)),
      Box::new(
        FixedWindowRollerBuilder::default()
          .build(&format!("{}.{{}}", self.file_name), 200)
          .unwrap(),
      ),
    );

    let full = RollingFileAppender::builder()
      .encoder(Box::new(JsonEncoder::new()))
      .build(&self.file_name, Box::new(policy))
      .unwrap();

    let config = Config::builder()
      .appender(Appender::builder().build("full", Box::new(full)))
      .logger(Logger::builder().build("rustls", LevelFilter::Trace))
      .logger(Logger::builder().build("ureq", LevelFilter::Trace))
      .build(Root::builder().appender("full").build(LevelFilter::Info))
      .unwrap();

    let _ = log4rs::init_config(config);
  }

  pub fn start_marker(&self) -> Marker {
    Marker::new(self)
  }
}

pub struct Marker {
  flag: String,
  file_name: String,
}

impl Marker {
  pub fn new(log: &AuditLog) -> Self {
    let flag = format!("{}", rand::random::<u64>());
    info!("start_marker_{}", flag);
    Self {
      flag,
      file_name: log.file_name.clone(),
    }
  }

  pub fn extract(self) -> String {
    use itertools::chain;
    use std::io::{BufRead, BufReader};

    info!("end_marker_{}", self.flag);

    let log = |name: &str| File::open(name).map(|f| BufReader::new(f).lines());
    let mut captured = String::new();
    let mut capture = false;

    for line in chain(log(&format!("{}.0", self.file_name)), log(&self.file_name)).flatten() {
      if let Ok(text) = line {
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        let message = json.get("message").unwrap().as_str().unwrap();

        if message == &format!("end_marker_{}", self.flag) {
          break;
        }
        if capture {
          captured.push_str(&text);
          captured.push('\n');
        }
        if message == &format!("start_marker_{}", self.flag) {
          capture = true;
        }
      }
    }

    captured
  }
}

describe! {
  test!{ creates_an_audit_log_and_captures_some_of_it
    let _ = std::process::Command::new("rm").args(&["-rf", "full.log.*"]).output();

    let log = AuditLog::new("/tmp/full.log".to_string(), 1200);
    log.start();
    info!("Hello");
    info!("World");
    let marker = log.start_marker();
    info!("Foo");
    info!("Bar");
    info!("Baz");
    let content = marker.extract();

    assert_eq!("Foo Bar Baz", content.lines()
      .map(|l|{
        serde_json::from_str::<serde_json::Value>(&l).unwrap()
          .get("message").unwrap()
          .as_str().unwrap()
          .to_string()
      })
      .collect::<Vec<String>>()
      .join(" "));

    info!("Goodbye World");
  }
}
