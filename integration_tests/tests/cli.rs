mod cli {
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

    api_integration_test!{ account_state(db, _chain)
      let alice = db.alice().await;
      alice.verify_email("alice@example.com").await;
      alice.write_signature_json_artifact();

      let mut json = run_command_json("account-state", &[]);
      assert_eq!(json.get("id").unwrap(), 1);
      /*
      json = run_command_json("account-state", &[]);
      assert_eq!(json.get("id").unwrap(), 1);
      */
    }

    api_integration_test!{ create_issuance_from_json(db, _chain)
      db.alice().await.write_signature_json_artifact();

      let json = run_command_json("create-issuance-from-json", &[
        "name_of_the_issuance",
        "--new-logo-text=testing",
        "--new-kind=badge",
        "--new-name=my_template",
      ]);

      println!("{}", &serde_json::to_string_pretty(&json)?);

      assert_eq!(json.get("templateName").unwrap(), "my_template");
      assert_eq!(json.get("templateKind").unwrap(), "BADGE");
    }

    fn run_command(command: &str, args: &[&str]) -> String {
      let params = [
        &["run", "-p", "constata-cli", "--","--config=target/artifacts/signature.json",
           "--password=password", command],
        args
      ].concat();

      let out = Command::new("cargo")
        .current_dir(std::fs::canonicalize("..").unwrap())
        .args(&params).output().unwrap();

      if !out.status.success() {
        println!("{}", std::str::from_utf8(out.stdout.as_slice()).unwrap());
        println!("{}", std::str::from_utf8(out.stderr.as_slice()).unwrap());
      }

      assert!(out.status.success());
      String::from_utf8(out.stdout).unwrap()
    }

    fn run_command_json(command: &str, args: &[&str]) -> serde_json::Value {
      serde_json::from_str(&run_command(command, args)).unwrap()
    }
  }
}
