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

    api_integration_test!{ create_issuances(db, mut _chain)
      db.alice().await.write_signature_json_artifact();

      //Create issuence 1 and check templateName
      assert_command("create-issuance-from-json", &[
        "name_of_the_issuance",
        "--new-logo-text=testing",
        "--new-kind=badge",
        "--new-name=my_template",],
        "/templateName", "my_template");

       assert_command("create-issuance-from-json", &[
        "2nd_name_of_the_issuance",
        "--new-logo-text=second",
        "--new-kind=diploma",
        "--new-name=2nd_template",],
        "/templateName", "2nd_template");
      
      assert_command("create-issuance-from-csv", &[
        "--csv-file",
        "integration_tests/static/cli_test_issuances_from_csv.csv",
        "first_csv_test_simpson",
        "-t",
        "2"],
        "/entriesCount", 10);

      //let issuance_state = run_command("issuance-state", &["1", "received"]);
      //assert_eq!(issuance_state, "true\n");
      assert_eq!(run_command("issuance-state", &["1", "received"]), "true\n");

      //let all_issuances_id_eq_2 = run_command_json("all-issuances", &["--id-eq", "2"]);
      //assert_eq!((&all_issuances_id_eq_2["allIssuances"][0]["templateId"]), 2);
      assert_command("all-issuances", &["--id-eq", "2"], "/allIssuances/0/templateId", 2);
      //assert_eq!((&all_issuances_id_eq_2["allIssuances"][0]["state"]), "received");
      assert_command("all-issuances", &["--id-eq", "2"], "/allIssuances/0/state", "received");

      db.site.request().create_all_received().await?; // Ahora se crean todos los documentos.

      //let all_issuances_id_eq_3 = run_command_json("all-issuances", &["--id-eq", "3"]);
      //assert_eq!((&all_issuances_id_eq_3["allIssuances"][0]["state"]), "created");
      assert_command("all-issuances", &["--id-eq", "3"], "/allIssuances/0/state", "created");

      run_command("preview-entry", &["9", "target/artifacts/cli_preview.html"]);

      run_command("sign-issuance", &["2"]);
      run_command("sign-issuance", &["3"]);

      //let all_issuances_id_eq_2_founded = run_command_json("all-issuances", &["--id-eq", "2"]);
      //assert_eq!((&all_issuances_id_eq_2_founded["allIssuances"][0]["state"]), "signed");
      assert_command("all-issuances", &["--id-eq", "2"], "/allIssuances/0/state", "signed");

      //let all_issuances_id_eq_empty = run_command_json("all-issuances", &["--id-eq", "21"]);
      //assert_eq!((all_issuances_id_eq_empty["allIssuances"][0]), Null);
      assert_none("all-issuances", &["--id-eq", "21"], "/allIssuances/0");

      _chain.fund_signer_wallet();
      _chain.simulate_stamping().await;
      db.site.request().try_complete().await?;
      
      //let prueba = run_command("issuance-export", &["3"]);
      assert_command("issuance-export", &["3"], "/id", 3);


      //let all_issuances_id_eq_2_completed = run_command_json("all-issuances", &["--id-eq", "2"]);
      //assert_eq!((&all_issuances_id_eq_2_completed["allIssuances"][0]["state"]), "completed");
      assert_command("all-issuances", &["--id-eq", "2"], "/allIssuances/0/state", "completed");

      //let all_issuances_name_like = run_command_json("all-issuances", &["--name-like", "simpson"]);
      //assert_eq!((all_issuances_name_like["allIssuances"][0]["name"]), "first_csv_test_simpson");
      assert_command("all-issuances", &["--name-like", "simpson"], "/allIssuances/0/name", "first_csv_test_simpson");

      run_command("entry-html-export", &["4", "target/artifacts/cli_entry_export.html"]);
      run_command("all-entries-html-export", &["target/artifacts"]);
            
      //assert_eq!((issuance_2["templateKind"]), "DIPLOMA");
      //assert_eq!((&all_issuances_id_eq_2["allIssuances"][0]["templateId"]), 2);
      //assert_eq!((&all_issuances_id_eq_2["allIssuances"][0]["state"]), "received");
      //assert_eq!((&all_issuances_id_eq_3["allIssuances"][0]["state"]), "created");
      //assert_eq!((&all_issuances_id_eq_2_founded["allIssuances"][0]["state"]), "signed");
      //assert_eq!((&all_issuances_id_eq_2_completed["allIssuances"][0]["state"]), "completed");
      //assert_eq!((all_issuances_id_eq_empty["allIssuances"][0]), Null);
      //assert_eq!((all_issuances_name_like["allIssuances"][0]["name"]), "first_csv_test_simpson");
    }

    api_integration_test!{ create_attestations(db, mut _chain)
      db.alice().await.write_signature_json_artifact();

      //let create_attestation  = run_command_json("create-attestation", &["-p", "integration_tests/static/id_example.jpg", "-m", "John Doe"]);
      //assert_eq!((&create_attestation["id"]), 1);
      assert_command("create-attestation", &["-p", "integration_tests/static/id_example.jpg", "-m", "John Doe"], "/id", 1);

      //let create_attestation_2 = run_command_json("create-attestation", &["-p", "integration_tests/static/id_example_2.jpg", "-m", "Bart"]);
      //assert_eq!((&create_attestation_2["id"]), 2);
      assert_command("create-attestation", &["-p", "integration_tests/static/id_example_2.jpg", "-m", "Bart"], "/id", 2);


      //let all_attestations_id_eq_2 = run_command_json("all-attestations", &["--id-eq", "2"]);
      //assert_eq!((&all_attestations_id_eq_2["allAttestations"][0]["state"]), "processing");
      assert_command("all-attestations", &["--id-eq", "2"], "/allAttestations/0/state", "processing");


      _chain.fund_signer_wallet();
      _chain.simulate_stamping().await;
      
      //let all_attestations_id_eq_2_done = run_command_json("all-attestations", &["--id-eq", "2"]);
      //assert_eq!((&all_attestations_id_eq_2_done["allAttestations"][0]["state"]), "done");
      assert_command("all-attestations", &["--id-eq", "2"], "/allAttestations/0/state", "done");


      //let attestations_state  = run_command_json("all-attestations", &[]);
      //println!("{} <= attestations state", &serde_json::to_string_pretty(&attestations_state)?);
      assert_command("all-attestations", &[], "/_allAttestationsMeta/count", 2);

      //let all_attestations_markers_like = run_command_json("all-attestations", &["--markers-like", "John"]);
      //println!("{} <======== all attestations markers like ", &serde_json::to_string_pretty(&all_attestations_markers_like)?);
      assert_command("all-attestations", &["--markers-like", "John"], "/allAttestations/0/markers", "John Doe");


      //let all_attestations_markers_like_empty = run_command_json("all-attestations", &["--markers-like", "nasa"]);
      assert_none("all-attestations", &["--markers-like", "nasa"], "/allAttestations/0");
      
      assert_command("attestation-html-export", &["2"], "/attestation/id", 2);     

      //println!("{}" , &serde_json::to_string_pretty(&create_attestation)?);
      //println!("{}", &serde_json::to_string_pretty(&create_attestation_2)?);
      //println!("{}", &serde_json::to_string_pretty(&attestations_state)?);
      //println!("This is Markers Like: q{}", &serde_json::to_string_pretty(&all_attestations_markers_like)?);
      //assert_eq!((&create_attestation["id"]), 1);
      //assert_eq!((&create_attestation_2["id"]), 2);
      //assert_eq!((&create_attestation["markers"]), "John Doe");
      
      //assert_eq!((&all_attestations_id_eq_2["allAttestations"][0]["state"]), "processing");
      //assert_eq!((&all_attestations_id_eq_2_done["allAttestations"][0]["state"]), "done");

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

    fn assert_command<T>(command: &str, args: &[&str], pointer: &str, expected_value: T)
      where 
        T: std::fmt::Debug + 'static,
        for<'a> &'a serde_json::Value: PartialEq<T>
    {
      assert_eq!(
        run_command_json(command, args).pointer(pointer).expect(&format!("Nothing found on {pointer}")),
        expected_value
      )
    }

    fn assert_none(command: &str, args: &[&str], pointer: &str) {
      assert!(run_command_json(command, args).pointer(pointer).is_none());
    }
  }
}
