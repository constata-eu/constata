constata_lib::describe_one! {
  use integration_tests::*;
  use assert_cmd::Command;
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

  api_integration_test!{ cli_account_state(db, _chain)
    let config = db.alice().await.write_signature_json_artifact();

    let out = Command::new("cargo")
      .current_dir(std::fs::canonicalize("..").unwrap())
      .args(&[
        "run",
        "-p",
        "constata-cli",
        "--",
        &format!("--config={config}"),
        "--password=password",
        "account-state",
      ])
      .output()?;

    assert!(out.status.success());
    assert_that!(&String::from_utf8(out.stdout).unwrap(), rematch("\"id\": 1"));
  }

  api_integration_test!{ cli_create_issuance_from_json(db, _chain)
    let config = db.alice().await.write_signature_json_artifact();
    let out = Command::new("cargo")
      .current_dir(std::fs::canonicalize("..").unwrap())
      .args(&[
        "run",
        "-p",
        "constata-cli",
        "--",
        &format!("--config={config}"),
        "--password=password",
        "create-issuance-from-json",
        "name_of_the_issuance",
        "--new-logo-text=testing",
        "--new-kind=diploma",
      ])
      .output()?;

    println!("{}", &String::from_utf8(out.stderr).unwrap());
    println!("{}", &String::from_utf8(out.stdout).unwrap());
    todo!("fail here");
    //assert!(out.status.success());
    //assert_that!(&String::from_utf8(out.stdout).unwrap(), rematch("\"org_id\": 2"));
  }
}
