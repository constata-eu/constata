use std::process::{Child, Command, Stdio};
use std::io::prelude::*;

pub struct PublicApiServer(Child);

impl PublicApiServer {
  pub fn start() -> Self {
    Command::new("killall")
      .args(&["-9", "public_api"])
      .output().expect("Could not kill previous server");

    let args = if std::env::var("CI").is_ok() {
      vec!["run", "--release", "-p", "public_api"]
    } else {
      vec!["run", "-p", "public_api"]
    };

    let path_to_log = "/tmp/constata_public_api_server.log";
    Command::new("rm").args(&["-rf", path_to_log]).output()
      .expect("Could not remove previous log");

    let output_file = std::fs::File::create(path_to_log).unwrap();

    let mut child = Command::new("cargo")
      .current_dir(std::fs::canonicalize("..").unwrap())
      .stdin(Stdio::piped())
      .stdout(Stdio::from(output_file.try_clone().unwrap()))
      .stderr(Stdio::from(output_file))
      .args(&args)
      .spawn().unwrap();

    let mut stdin = child.stdin.take().unwrap();
    std::thread::spawn(move || {
      stdin.write(b"password\n").unwrap();
      stdin.flush().unwrap();
    });

    for i in 0..100 {
      let status = ureq::get("http://localhost:8000").call();
      if status.is_ok() {
        break;
      }
      std::thread::sleep(std::time::Duration::from_millis(500));
      if i == 99 && std::env::var("CI").is_err() {
        assert!(false, "Public api server is taking too long. Try compiling separately and come back.");
      }
    }

    PublicApiServer(child)
  }

  pub fn stop(&mut self) {
    self.0.kill().unwrap();
  }
}

impl std::ops::Drop for PublicApiServer {
  fn drop(&mut self) {
    self.stop();
  }
}
