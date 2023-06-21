mod cli {
  constata_lib::describe_one! {
    use integration_tests::*;
    use assert_cmd::Command;

    api_integration_test!{ create_issuances(db, mut chain)
      db.alice().await.write_signature_json_artifact();

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

      assert_eq!(run_command("issuance-state", &["1", "received"]), "true\n");
      assert_command("all-issuances", &["--id-eq", "2"], "/allIssuances/0/templateId", 2);
      assert_command("all-issuances", &["--id-eq", "3"], "/allIssuances/0/state", "received");

      db.site.issuance().create_all_received().await?; // Ahora se crean todos los documentos.

      assert_command("all-issuances", &["--id-eq", "3"], "/allIssuances/0/state", "created");
      run_command("preview-entry", &["9", "target/artifacts/cli_preview.html"]);
      failed_command("preview-entry", &["12", "target/artifacts/cli_entry_export.html"]);
      run_command("sign-issuance", &["2"]);
      run_command("sign-issuance", &["3"]);

      assert_command("all-issuances", &["--id-eq", "3"], "/allIssuances/0/state", "signed");
      assert_none("all-issuances", &["--id-eq", "21"], "/allIssuances/0");

      chain.fund_signer_wallet();
      chain.simulate_stamping().await;
      db.site.issuance().try_complete().await?;
      
      assert_command("issuance-export", &["3"], "/id", 3);
      assert_command("all-issuances", &["--id-eq", "2"], "/allIssuances/0/state", "completed");
      assert_command("all-issuances", &["--id-eq", "3"], "/allIssuances/0/state", "completed");
      assert_command("all-issuances", &["--name-like", "simpson"], "/allIssuances/0/name", "first_csv_test_simpson");
      run_command("entry-html-export", &["4", "target/artifacts/cli_entry_export.html"]);
      failed_command("entry-html-export", &["20", "target/artifacts/cli_entry_export.html"]);
      run_command("all-entries-html-export", &["target/artifacts"]);
            
    }

    api_integration_test!{ create_attestations(db, mut chain)
      db.alice().await.write_signature_json_artifact();

      assert_command("create-attestation", &[
        "-p",
        "integration_tests/static/id_example.jpg",
        "-m",
        "John Doe"],
        "/id", 1);

      assert_command("create-attestation", &[
        "-p",
        "integration_tests/static/id_example_2.jpg",
        "-m",
        "Bart"],
        "/personId", 1);
        
      assert_command("all-attestations", &["--id-eq", "2"], "/allAttestations/0/state", "processing");

      chain.fund_signer_wallet();
      chain.simulate_stamping().await;
      
      assert_command("all-attestations", &["--id-eq", "2"], "/allAttestations/0/state", "done");
      assert_command("all-attestations", &[], "/_allAttestationsMeta/count", 2);
      assert_command("all-attestations", &["--markers-like", "John"], "/allAttestations/0/markers", "John Doe");
      assert_none("all-attestations", &["--markers-like", "nasa"], "/allAttestations/0");
      assert_command("attestation-html-export", &["2"], "/attestation/id", 2);     

      assert_none("all-attestations", &["--id-eq", "2"], "/allAttestations/0/publicCertificateUrl");
      assert!(
        run_command_json("attestation-set-published", &["2", "--publish"])
          .pointer("/publicCertificateUrl").unwrap().is_string()
      );

      assert_none("attestation-set-published", &["2"], "/publicCertificateUrl");
    }

    api_integration_test!{ account_state(db, _chain)
      let alice = db.alice().await;
      alice.verify_email("alice@example.com").await;
      alice.write_signature_json_artifact();

      let json = run_command_json("account-state", &[]);
      assert_eq!(json.get("id").unwrap(), 1);
    }

    
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
      let response = run_command_json(command, args);
      assert_eq!(
        response.pointer(pointer).expect(&format!("Nothing found on {pointer} for response {response}")),
        expected_value
      )
    }

    fn assert_none(command: &str, args: &[&str], pointer: &str) {
      let result = run_command_json(command, args);
      assert!(matches!(result.pointer(pointer), None | Some(serde_json::Value::Null)));
    }

    fn failed_command(command: &str, args: &[&str]) -> String {
      let params = [
        &["run", "-p", "constata-cli", "--","--config=target/artifacts/signature.json",
           "--password=password", command],
        args
      ].concat();

      let out = Command::new("cargo")
        .current_dir(std::fs::canonicalize("..").unwrap())
        .args(&params).output().unwrap();

      if out.status.success() {
        println!("{}", std::str::from_utf8(out.stdout.as_slice()).unwrap());
        println!("{}", std::str::from_utf8(out.stderr.as_slice()).unwrap());
      }

      assert!(!out.status.success());
      String::from_utf8(out.stderr).unwrap()
    }
  }
}
