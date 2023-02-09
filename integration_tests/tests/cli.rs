//use assert_cmd::Command;

constata_lib::describe_one! {
  /*
   *  This test is commented until we have a new release.
  test!{ cli_standard_workflow
    TestDb::new().await?;
    let _chain = TestBlockchain::new().await;
    let mut server = public_api_server::PublicApiServer::start();

    let download_url = format!(
      "https://github.com/constata-eu/constata-client/releases/download/rc-3/constata-cli-{}",
      std::env::consts::OS
    );

    Command::new("curl").args(&["-L", "-o", "/tmp/constata-cli", &download_url]).output()
      .expect("failed to download constata-cli");

    Command::new("chmod").args(&["+x", "/tmp/constata-cli"]).output()
      .expect("failed to give execution permissions");

    Command::new("/tmp/constata-cli")
      .arg("--config=constata_conf_cli.json")
      .arg("--password=not_so_secret")
      .arg("stamp")
      .arg("constata_conf_cli.json")
      .assert()
      .success();

    server.stop();
  }
  */
}
