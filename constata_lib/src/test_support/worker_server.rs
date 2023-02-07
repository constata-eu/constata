use std::process::{Child, Command, Stdio};
use std::io::prelude::*;

pub struct WorkerServer(Child);

impl WorkerServer {
  pub fn start() -> Self {
    Command::new("killall")
      .args(&["-9", "worker"])
      .output().expect("Could not kill previous server");

    let mut child = Command::new("cargo")
      .current_dir(std::fs::canonicalize("../worker").unwrap())
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .args(&["run", "-p", "worker"])
      .spawn().unwrap();

    if let Some(_status) = child.try_wait().expect("no puede intentar esperar al worker") {
      panic!("No pude levantar el worker");
    }

    let mut stdin = child.stdin.take().unwrap();
    std::thread::spawn(move || {
      stdin.write(b"password\n").unwrap();
      stdin.flush().unwrap();
    });

    WorkerServer(child)
  }

  pub fn stop(&mut self) {
    self.0.kill().unwrap();
  }
}

impl std::ops::Drop for WorkerServer {
  fn drop(&mut self) {
    self.stop();
  }
}
