mod cli {
  constata_lib::describe_one! {
    use integration_tests::*;
    use assert_cmd::Command;
    use serde_json::Value::Null;


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

    api_integration_test!{ create_issuances(db, mut _chain)
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

      let create_issuance_from_csv  = run_command_json("create-issuance-from-csv", &["--csv-file", "integration_tests/static/cli_test_issuances_from_csv.csv", "first_csv_test_simpson", "-t", "2"]);
      let issuance_state = run_command("issuance-state", &["1", "received"]);
      let all_issuances_id_eq_2 = run_command_json("all-issuances", &["--id-eq", "2"]);
            
      db.site.request().create_all_received().await?; // Ahora se crean todos los documentos.

      let all_issuances_id_eq_3 = run_command_json("all-issuances", &["--id-eq", "3"]);

      run_command("preview-entry", &["9", "target/artifacts/cli_preview.html"]);
      run_command("sign-issuance", &["2"]);

      run_command("sign-issuance", &["3"]);

      let all_issuances_id_eq_2_founded = run_command_json("all-issuances", &["--id-eq", "2"]);
      let all_issuances_id_eq_empty = run_command_json("all-issuances", &["--id-eq", "21"]);

      _chain.fund_signer_wallet();
      _chain.simulate_stamping().await;
      db.site.request().try_complete().await?;
      
      run_command("issuance-export", &["3", "target/artifacts/cli_issuance_export.csv"]);

      let all_issuances_id_eq_2_completed = run_command_json("all-issuances", &["--id-eq", "2"]);
      let all_issuances_name_like = run_command_json("all-issuances", &["--name-like", "simpson"]);

      run_command("entry-html-export", &["4", "target/artifacts/cli_entry_export.html"]);
            
      assert_eq!((&issuance_1["templateName"]), "my_template");
      assert_eq!((issuance_2["templateKind"]), "DIPLOMA");
      assert_eq!((&all_issuances_id_eq_2["allIssuances"][0]["templateId"]), 2);
      assert_eq!((&all_issuances_id_eq_2["allIssuances"][0]["state"]), "received");
      assert_eq!((&all_issuances_id_eq_3["allIssuances"][0]["state"]), "created");
      assert_eq!((&all_issuances_id_eq_2_founded["allIssuances"][0]["state"]), "signed");
      assert_eq!((&all_issuances_id_eq_2_completed["allIssuances"][0]["state"]), "completed");
      assert_eq!((create_issuance_from_csv["entriesCount"]), 10);
      assert_eq!(issuance_state, "true\n");
      assert_eq!((all_issuances_id_eq_empty["allIssuances"][0]), Null);
      assert_eq!((all_issuances_name_like["allIssuances"][0]["name"]), "first_csv_test_simpson");
    }

    api_integration_test!{ create_attestations(db, mut _chain)
      db.alice().await.write_signature_json_artifact();

      let create_attestation  = run_command_json("create-attestation", &["-p", "integration_tests/static/id_example.jpg", "2023-04-29T19:33:45.762762Z", "John Doe"]);

      let create_attestation_2 = run_command_json("create-attestation", &["-p", "integration_tests/static/id_example_2.jpg", "2023-04-30T19:33:45.762762Z", "Bart"]);

      let all_attestations_id_eq_2 = run_command_json("all-attestations", &["--id-eq", "2"]);

      _chain.fund_signer_wallet();
      _chain.simulate_stamping().await;

      let all_attestations_id_eq_2_done = run_command_json("all-attestations", &["--id-eq", "2"]);
     
      let attestations_state  = run_command_json("all-attestations", &[]);

      let all_attestations_markers_like = run_command_json("all-attestations", &["--markers-like", "John"]);

      println!("{}" , &serde_json::to_string_pretty(&create_attestation)?);
      println!("{}", &serde_json::to_string_pretty(&create_attestation_2)?);
      println!("{}", &serde_json::to_string_pretty(&attestations_state)?);
      println!("This is Markers Like: q{}", &serde_json::to_string_pretty(&all_attestations_markers_like)?);
      assert_eq!((&create_attestation["id"]), 1);
      assert_eq!((&create_attestation_2["id"]), 2);
      assert_eq!((&create_attestation["markers"]), "John Doe");
      assert_eq!((&all_attestations_id_eq_2["allAttestations"][0]["state"]), "processing");
      assert_eq!((&all_attestations_id_eq_2_done["allAttestations"][0]["state"]), "done");
    }

    api_integration_test!{ account_state(db, _chain)
      let alice = db.alice().await;
      alice.verify_email("alice@example.com").await;
      alice.write_signature_json_artifact();

      let json = run_command_json("account-state", &[]);
      assert_eq!(json.get("id").unwrap(), 1);

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

      run_command("--help", &[]);
     
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
