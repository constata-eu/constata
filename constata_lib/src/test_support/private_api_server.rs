use std::process::{Child, Command, Stdio};
use std::io::prelude::*;

pub struct PrivateApiServer(Child);

impl PrivateApiServer {
  pub fn start() -> Self {
    Command::new("killall")
      .args(&["-9", "private_api"])
      .output().expect("Could not kill previous server");

    let path_to_log = "/tmp/constata_private_api_server.log";
    Command::new("rm").args(&["-rf", path_to_log]).output()
      .expect("Could not remove previous log");

    let output_file = std::fs::File::create(path_to_log).unwrap();

    let mut child = Command::new("cargo")
      .current_dir(std::fs::canonicalize("../private_api").unwrap())
      .stdin(Stdio::piped())
      .stdout(Stdio::from(output_file.try_clone().unwrap()))
      .stderr(Stdio::from(output_file))
      .args(&["run", "-p", "private_api"])
      .spawn().unwrap();

    let mut stdin = child.stdin.take().unwrap();
    std::thread::spawn(move || {
      stdin.write(b"password\n").unwrap();
      stdin.flush().unwrap();
    });

    loop {
      let status = ureq::get("http://localhost:8000").call();
      if let Err(ureq::Error::Status(404, _)) = status {
        break;
      }
    }

    PrivateApiServer(child)
  }

  pub fn stop(&mut self) {
    self.0.kill().unwrap();
  }
}

impl std::ops::Drop for PrivateApiServer {
  fn drop(&mut self) {
    self.stop();
  }
}
