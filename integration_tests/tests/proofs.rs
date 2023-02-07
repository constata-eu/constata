mod proof_integration {
  constata_lib::describe_one! {
    use bitcoin::network::constants::Network;
    use integration_tests::*;
    use thirtyfour::{
      extensions::cdp::{ChromeDevTools, NetworkConditions},
    };

    integration_test!{ proofs_integration_test (c, d)
      let mut chain = TestBlockchain::new().await;
      let key = TestBlockchain::default_private_key().await.unwrap();

      let alice = c.alice().await;
      alice.make_pubkey_domain_endorsement().await;
      alice.make_kyc_endorsement().await;

      let story = c.alice().await.add_funds().await
        .story_with_signed_doc(&read("document.zip"), None, "").await;

      chain.fund_signer_wallet();
      chain.simulate_stamping().await;

      c.bot().await.witnessed_email(&story, samples::multipart_email().as_bytes(), None).await;
      chain.simulate_stamping().await;

      let content = story.proof(Network::Regtest, &key).await?.render_html(i18n::Lang::Es).unwrap();

      let content_path = "/tmp/content.html";
      std::fs::write(&content_path, &content).unwrap();

      d.goto(&format!("file://{}", content_path)).await;
      d.wait_for("#document_0 .previews .preview img").await;

      d.click("#document_0 .document-index .field-1 .link-save").await;
      d.check_downloads_for_file("1_extras_photo.jpg").await;

      // Assertion: Shows network-error message when no internet.
      let dev_tools = ChromeDevTools::new(d.driver.handle.clone());
      let mut conditions = NetworkConditions::new();
      conditions.offline = true;
      dev_tools.set_network_conditions(&conditions).await?;
      d.goto(&format!("file://{}", content_path)).await;
      d.wait_for_text("#message h1", r"NO SE PUDO VALIDAR EL CERTIFICADO.*").await;
      conditions.offline = false;
      dev_tools.set_network_conditions(&conditions).await?;

      let corrupt_path = "/tmp/corrupt_content.html";
      std::fs::write(&corrupt_path, &content.replace("9f167c730f2d9eac8c187c6b2654b1860a4e4719b9f35916857e937acc25ea46", "ABC1")).unwrap();
      d.goto(&format!("file://{}", corrupt_path)).await;
      d.wait_for_text("#message h1", r"CERTIFICADO INVÁLIDO.*").await;


      d.goto("http://localhost:8000/safe").await;
      d.fill_in("#certificate", corrupt_path).await;
      d.wait_for("#invalid-certificate").await;
      
      d.fill_in("#certificate", content_path).await;
      d.wait_for("#iframe-valid-certificate").await.enter_frame().await.expect("to enter frame");
      d.wait_for("#document_0").await;

      d.click("#expand_audit_log").await;
      d.click("#validate_document_1_part_2 a").await;
      d.check_downloads_for_file("2_hello.txt").await;
    }

    test!{ can_generate_for_realistic_email 
      let c = TestDb::new().await.unwrap();
      let mut chain = TestBlockchain::new().await;
      let mut server = public_api_server::PublicApiServer::start();
      let selenium = Selenium::start().await;

      let bot = c.bot().await.accept_terms_and_conditions().await;
      let document = bot.witnessed_email_with_story_and_payload(
        &std::fs::read("tests/resources/multipart_email_with_text_and_attachment").unwrap(),
        Some(("message/rfc822".to_string(), "email".to_string())),
      ).await;

      let key = TestBlockchain::default_private_key().await?;

      chain.fund_signer_wallet();
      chain.simulate_stamping().await;

      let content = document.story().await?.proof(Network::Regtest, &key).await?
        .render_html(i18n::Lang::Es).expect("Content to be ready now");

      let content_path = "/tmp/content.html";
      std::fs::write(&content_path, &content).unwrap();
      let d = &selenium.driver;

      d.goto(&format!("file://{}", content_path)).await?;

      server.stop();
      selenium.stop().await;
    }

    integration_test!{ makes_certificate_for_email_conversation (c, d)
      let mut chain = TestBlockchain::new().await;

      let alice = c.alice().await;
      alice.make_pubkey_domain_endorsement().await;
      alice.make_kyc_endorsement().await;
      let bot = c.bot().await.accept_terms_and_conditions().await;

      let story = alice.clone().add_funds().await.story_with_signed_doc(&read("document.zip"), None, "").await;
      let doc = &story.documents().await?[0];

      chain.fund_signer_wallet();
      chain.simulate_stamping().await;

      let token = alice.make_download_proof_link_from_doc(&doc, 30).await.token().await?;

      bot.witnessed_email(&story, samples::multipart_email().as_bytes(), None).await;

      d.goto(&format!("http://localhost:8000/safe/{token}")).await;
      d.wait_for("#pending_docs_title").await;

      chain.simulate_stamping().await;

      d.driver.refresh().await?;
      d.wait_until_gone("#pending_docs_title").await;
    }

    integration_test!{ makes_certificate_for_a_single_document (c, d)
      let mut chain = TestBlockchain::new().await;

      let alice = c.alice().await;
      alice.make_pubkey_domain_endorsement().await;
      alice.make_kyc_endorsement().await;

      let story = alice.clone().add_funds().await.story_with_signed_doc(
        b"\0\0\0hello",
        None,
        ""
      ).await;
      let doc = &story.documents().await?[0];

      chain.fund_signer_wallet();
      chain.simulate_stamping().await;

      let token = alice.make_download_proof_link_from_doc(&doc, 30).await.token().await?;

      d.goto(&format!("http://localhost:8000/safe/{token}/show")).await;
      d.wait_for("#iframe-valid-certificate").await.enter_frame().await.expect("to enter frame");

      d.wait_for_text(".meta-section p strong", r#"application/octet-stream"#).await;
    }

  /*
    test!{ offers_viewing_download_proof_link_directly_in_safe_env
      let c = TestDb::new().await.unwrap();
      let mut chain = TestBlockchain::new().await;
      let mut server = public_api_server::PublicApiServer::start();
      let selenium = Selenium::start().await;
      let d = &selenium.driver;

      let bot = c.bot().await.accept_terms_and_conditions().await;
      let document = bot.witnessed_email_with_story_and_payload(
        &std::fs::read("tests/resources/multipart_email_with_text_and_attachment").unwrap(),
        Some(("message/rfc822".to_string(), "email".to_string())),
      ).await;

      chain.fund_signer_wallet();
      chain.simulate_stamping().await;

      let token = document.create_download_proof_link(30).await?.attrs.token;

      d.goto(&format!("http://localhost:8000/safe/{token}")).await?;

      // Assertion: Can download an attachment.
      let elem = d.query(By::Css(".btn-download")).first().await?;
      elem.wait_until().clickable().await?;
      elem.click().await?;

      /*
      d.in_new_tab(|| async {
        d.goto("chrome://downloads/").await.unwrap();
        d.query(By::Css("downloads-manager")).first().await.unwrap()
          .get_shadow_root().await.unwrap()
          .query(By::Css("downloads-item")).first().await.unwrap()
          .get_shadow_root().await?.query(By::Css("a"))
          .with_text(StringMatch::new("certificado").partial())
          .first().await.expect("Link matcher to be correct")
          .wait_until().displayed().await
      }).await?;
      */

      // Assertion: Can download an attachment.
      let show_link = d.query(By::Css(".btn-show")).first().await?;
      show_link.wait_until().clickable().await?;
      show_link.click().await?;

      d.query(By::Css("#message"))
        .with_text(StringMatch::new("Este certificado no puede verse").partial())
        .first().await.expect("Link matcher to be correct")
        .wait_until().displayed().await?;

      d.goto(&format!("http://localhost:8000/safe/bogustoken")).await?;
      d.query(By::Css("h1"))
        .with_text(StringMatch::new("El link de descarga no existe o expiró").partial())
        .first().await.expect("Link matcher to be correct")
        .wait_until().displayed().await?;

      server.stop();
      selenium.stop().await;
    }*/
  }
}
