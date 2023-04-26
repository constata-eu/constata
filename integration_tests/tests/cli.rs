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

    api_integration_test!{ create_issuance_from_json(db, mut _chain)
      db.alice().await.write_signature_json_artifact();

      let issuance_1 = run_command_json("create-issuance-from-json", &[
        "name_of_the_issuance",
        "--new-logo-text=testing",
        "--new-kind=badge",
        "--new-name=my_template",
      ]);

      let issuance_2 = run_command_json("create-issuance-from-json", &[
        "2nd_name_of_the_issuance",
        "--new-logo-text=second",
        "--new-kind=diploma",
        "--new-name=2nd_template",
      ]);

      let create_issuance_from_csv  = run_command_json("create-issuance-from-csv", &["--csv-file", "integration_tests/static/cli_test_issuances_from_csv.csv", "first_csv_test", "-t", "2"]);

      let all_issuances = run_command_json("all-issuances", &[]);

      let issuance_state = run_command("issuance-state", &["1", "received"]);

      let all_issuances_id_2 = run_command_json("all-issuances", &["--ids", "2"]);
            
      db.site.request().create_all_received().await?; // Ahora se crean todos los documentos.

      let all_issuances_id_3 = run_command_json("all-issuances", &["--ids", "3"]);

      run_command("preview-entry", &["9", "target/artifacts/cli_preview.html"]);

      run_command("sign-issuance", &["2"]);

      run_command("sign-issuance", &["3"]);

      let all_issuances_id_2_founded = run_command_json("all-issuances", &["--ids", "2"]);

      _chain.fund_signer_wallet();
      _chain.simulate_stamping().await;
  
      db.site.request().try_complete().await?;
      
      run_command("issuance-export", &["3", "target/artifacts/cli_issuance_export.csv"]);

      let all_issuances_id_2_completed = run_command_json("all-issuances", &["--ids", "2"]);

      let all_issuances_after_sign_and_stamping = run_command_json("all-issuances", &[]);

      

      //let all_templates = run_command_json("all-templates", &[]);
      
      println!("{}", &serde_json::to_string_pretty(&issuance_1)?);

      println!("{}", &serde_json::to_string_pretty(&all_issuances)?);

      println!("{}", (&issuance_state));

      //println!("{}", &serde_json::to_string_pretty(&issuance_state)?);
      println!("{}", &serde_json::to_string_pretty(&all_issuances_id_2)?);
      println!("{}", &serde_json::to_string_pretty(&all_issuances_id_2_founded)?);
      println!("{}", &serde_json::to_string_pretty(&all_issuances_id_2_completed)?);
      println!("{}", &serde_json::to_string_pretty(&create_issuance_from_csv)?);
      println!("{}", &serde_json::to_string_pretty(&all_issuances_after_sign_and_stamping)?);
      //println!("{}", &serde_json::to_string_pretty(&all_templates)?);
      //println!("this is the CHAIN: {:?}", (&chain_output));


      assert_eq!((&issuance_1["templateName"]), "my_template");
      assert_eq!((issuance_2["templateKind"]), "DIPLOMA");
      assert_eq!((&all_issuances_id_2["allIssuances"][0]["templateId"]), 2);
      assert_eq!((&all_issuances_id_2["allIssuances"][0]["state"]), "received");
      assert_eq!((&all_issuances_id_3["allIssuances"][0]["state"]), "created");
      assert_eq!((&all_issuances_id_2_founded["allIssuances"][0]["state"]), "signed");
      assert_eq!((&all_issuances_id_2_completed["allIssuances"][0]["state"]), "completed");
      assert_eq!((create_issuance_from_csv["entriesCount"]), 10);
      assert_eq!(issuance_state, "true\n");
      
    }

    api_integration_test!{ account_state(db, _chain)
      let alice = db.alice().await;
      alice.verify_email("alice@example.com").await;
      alice.write_signature_json_artifact();

      let json = run_command_json("account-state", &[]);
      assert_eq!(json.get("id").unwrap(), 1);

      println!("{}", &serde_json::to_string_pretty(&json)?);
      /*
      json = run_command_json("account-state", &[]);
      assert_eq!(json.get("id").unwrap(), 1);
      */
    }

    /*
    api_integration_test!{ all_issuances(db, _chain)
      let _alice = db.alice().await;

      let json = run_command_json("all-issuances", &[]);
      //assert_eq!(json.get("id").unwrap(), 1);

      println!("{}", &serde_json::to_string_pretty(&json)?);

    }
    */
    api_integration_test!{ help(db, _chain)
      db.alice().await.write_signature_json_artifact();

      let no_json = run_command("--help", &[]);

      println!("{}", (&no_json));
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
