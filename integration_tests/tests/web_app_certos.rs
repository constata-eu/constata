mod website {
  constata_lib::describe_one! {
    use integration_tests::*;
    use constata_lib::{
      Site,
      models::{
        KycRequestProcessForm,
        template::InsertTemplate,
        template_kind::TemplateKind,
        download_proof_link::DownloadProofLink,
      },
      Result,
    };
    use bitcoin::network::constants::Network;
    use std::env;

    integration_test!{ full_flow_from_signup_until_stamped (c, d)
      signup_and_verify(&d, &c.site).await;
      create_wizard(&d, &c.site, 2, "testing-template").await;
      d.click("#preview-1").await;

      let handles = d.get_handles_and_go_to_window_one().await;
      d.click("button").await;
      d.driver.switch_to_window(handles[0].clone()).await?;

      sign_wizard(&d).await;

      d.click("#dashboard-menu-item").await;
      d.wait_for_text("h2", "In progress").await;
    }
 
    integration_test!{ issues_diplomas_from_csv_and_completes_them (c, d)
      let mut chain = TestBlockchain::new().await;

      signup(&d).await;
      for _ in 0..4 {
        create_template(&d, "testing-template", "DIPLOMA").await;
        let csv = format!("{}/tests/resources/default_certos_recipients.csv", env::current_dir().unwrap().display());
        add_recipients_with_csv(&d, &c.site, &csv).await;
        sign_wizard(&d).await;
        d.click("a[href='#/']").await;
      }
      d.click("#dashboard-menu-item").await;
      d.wait_for_text("h2", "In progress").await;
      chain.fund_signer_wallet();
      chain.simulate_stamping().await;
      try_until(40, "bulletin_is_published", || async { c.site.bulletin().find(1).await.unwrap().is_published() }).await;
      c.site.request().try_complete().await?;
      d.wait_for_text("h2", "Recent issuances").await;
      d.click("a[href='#/Issuance/4/show']").await;
      d.click("#export_to_csv").await;
      d.check_downloads_for_file("constata_issuance_4.csv").await;
    }

    integration_test!{ sign_previously_created_issuance (c, d)
      signup_and_verify(&d, &c.site).await;
      create_wizard(&d, &c.site, 2, "testing-template").await;
      reload(&d).await;
      d.click("a[href='#/wizard/1']").await;
      sign_wizard(&d).await;
    }

    integration_test!{ use_previous_template_to_create_issuances (c, d)
      signup_and_verify(&d, &c.site).await;
      create_wizard(&d, &c.site, 2, "testing-template").await;
      sign_wizard(&d).await;
      d.click("a[href='#/']").await;
      d.click("a[href='#/wizard']").await;
      d.click("#templateId").await;
      autoselect_first_option(&d).await;

      for i in 0..2 {
        let email = format!("probando{i}@constata.eu");
        add_recipient(&d, "Luciano Carreño", &email, "82736123", i).await;
      }

      d.click("#continue").await;
      d.wait_for("span[role='progressbar']").await;

      c.site.request().create_all_received().await.expect("to create sucessfully the entries");
      d.wait_for_text("h2", "Review and sign").await;
      sign_wizard(&d).await;
      d.click("a[href='#/']").await;
    }

    integration_test!{ create_custom_template_and_create_issuance (c, d)
      let mut chain = TestBlockchain::new().await;
      signup_and_verify(&d, &c.site).await;
      let payload = std::fs::read("static/custom_template.zip").expect("custom_template.zip");

      c.site.template().insert(InsertTemplate{
        app_id: 1,
        person_id: 1,
        org_id: 1,
        name: "Tempalte Custom".to_string(),
        kind: TemplateKind::Diploma,
        schema: d.template_custom_schema(),
        og_title_override: Some("Curso de programación".to_string()),
        custom_message: Some("Mensaje custom".to_string()),
        size_in_bytes: payload.len() as i32,
      }).validate_and_save(&payload).await?;
      
      d.click("a[href='#/wizard']").await;
      d.click("#templateId").await;
      autoselect_first_option(&d).await;

      for i in 0..2 {
        let name = format!("nombre_{i}");
        d.click("#recipients > button").await;
        d.fill_in("#name", &name).await;
        if i == 0 {
          d.fill_in("#curso", "Desarrollo Web").await;
          d.fill_in("#day", "22").await;
          d.fill_in("#month", "Diciembre").await;
          d.fill_in("#year", "2022").await;
        }
        d.fill_in("#nota_global", "10").await;
        d.click("button[type='submit']").await;
      }

      d.click("#continue").await;
      d.wait_for("span[role='progressbar']").await;

      c.site.request().create_all_received().await.expect("to create sucessfully the entries");
      d.wait_for_text("h2", "Review and sign").await;
      sign_wizard(&d).await;
      d.click("a[href='#/']").await;

      chain.fund_signer_wallet();
      chain.simulate_stamping().await;

      let alice = c.alice().await;
      let entry = c.site.entry().find(&1).await?;
      let doc = entry.document().await?.expect("entry's document");
      let token = alice.make_download_proof_link_from_doc(&doc, 30).await.token().await?;
      d.goto(&format!("http://localhost:8000/#/safe/{token}")).await;
      d.click("#safe-button-change-public-certificate-state").await;
      let title = "Curso de programación";
      let description = "Diploma issued by apps.script.testing@constata.eu via Constata.eu";
      let image = "https://constata.eu/assets/images/logo.png";
      check_public_certificate(&d, &title, &description, &image).await;
    }

    integration_test!{ discards_issuance (c, d)
      signup_and_verify(&d, &c.site).await;
      create_wizard(&d, &c.site, 2, "testing-template").await;
      d.wait_for("#issuances-menu-item").await;
      d.click("#discard-button").await;
      d.click("#confirm-discard-button").await;
      d.wait_for("#template-wizard-step").await;
      d.click("#issuances-menu-item").await;
      d.wait_for_text("p", "No results found").await;
    }

    integration_test!{ login_and_logout (c, d)
      let path = signup_and_verify(&d, &c.site).await;
      d.click("#logout-menu-item").await;
      d.wait_for_text("h1", "Hello again!").await; 

      d.click("#button_use_another_signature").await;
      d.click(".ra-confirm").await;
      d.wait_for_text("h1", "Hello").await; 

      d.fill_in("input[type='file']", &path).await;
      d.wait_for_text("h1", "Hello again!").await; 

      d.fill_in("#password", "password").await;
      d.click("button[type='submit']").await;
      d.wait_for("#constata_dashboard").await;
      reload(&d).await;
    }

    integration_test!{ notifies_no_emails_will_be_sent_for_unverified_customers (_c, d)
      signup(&d).await;
      create_template(&d, "testing-template", "DIPLOMA").await;
      d.wait_for_text(".MuiAlert-message div", "Recipient notifications disabled").await;
    }

    integration_test!{ sends_kyc_request_and_accepts_it (c, d)
      signup_and_verify(&d, &c.site).await;
      create_kyc_request_and_process_it(&d, &c.site, "accept").await;
      reload(&d).await;
      d.wait_for_text("#section-endorsement-existing h2", r"Verified identity").await;
      check_autocomplete_in_kyc(&d).await;
    }

    integration_test!{ rejected_kyc_values_are_autocompleted_next_time (c, d)
      signup_and_verify(&d, &c.site).await;
      create_kyc_request_and_process_it(&d, &c.site, "reject").await;
      assert_that!(c.site.kyc_endorsement().find(&1).await.is_err());
      reload(&d).await;
      d.wait_for_text(".MuiPaper-elevation1 > div > h2", r"Unverified identity*").await;
      check_autocomplete_in_kyc(&d).await;
    }

    integration_test!{ verify_email_address_and_notify_if_already_in_use (c, d)
      signup(&d).await;

      let url = c.site.email_address().find(&1).await.expect("to have an email_address")
        .full_url().await.expect("to have a full url to verify email");

      d.goto(&url).await;
      d.wait_for_text(".MuiAlert-message", r"Email address verified*").await;
      d.wait_for("a[href='/']").await;

      d.goto(&url).await;
      d.wait_for(".MuiAlert-outlinedWarning").await;
      d.goto("/").await;
      d.click("#button_use_another_signature").await;
      d.click(".ra-confirm").await;
      d.wait_for_text("h1", "Hello").await; 
      fill_signup_form(&d).await;
      d.wait_for_text(".MuiAlert-message", r"This email address is in use*").await;
    }

    integration_test!{ change_email_after_registration_and_verify_it (c, d)
      signup(&d).await;
      change_email_address(&d, "otro.email@gmail.com", true).await;
      
      let person = c.site.person().find(&1).await?;
      let email = person.email_address().await?.expect("to have an email address at this instance.");
      assert_eq!(&email.attrs.address, "otro.email@gmail.com");
      c.site.email_address().verify_with_token(&email.access_token().await.unwrap().unwrap()).await.expect("to verify email address");

      assert_eq!(&person.org().await?.name_for_on_behalf_of().await?, "otro.email@gmail.com");
      assert_eq!(person.verified_email_address().await?.expect("to have a verified email address").address(), "otro.email@gmail.com");
      assert_eq!(person.email_address().await?.expect("to have an email address").address(), "otro.email@gmail.com");
      assert_eq!(person.last_email_address().await?.address(), "otro.email@gmail.com");

      change_email_address(&d, "apps.script.testing@constata.eu", false).await;
      assert_eq!(&person.org().await?.name_for_on_behalf_of().await?, "otro.email@gmail.com");
      assert_eq!(person.verified_email_address().await?.expect("to have a verified email address").address(), "otro.email@gmail.com");
      assert_eq!(person.email_address().await?.expect("to have an email address").address(), "apps.script.testing@constata.eu");
      assert_eq!(person.last_email_address().await?.address(), "apps.script.testing@constata.eu");
    }

    integration_test!{ buy_token_with_invoice_link (c, d)
      async fn check_i_am_in_buy_tokens_page(d: &Selenium, another_window: bool) {
        if another_window {
          d.get_handles_and_go_to_window_one().await;
        }
        d.fill_in("#amount", "4").await;
        d.wait_for_text("#pay-with-credit-card", r"Pay with Credit Card*").await;
        d.wait_for_text("#pay-with-bitcoin", r"Pay with Bitcoin*").await;
        d.wait_for("#invoice-link-buy button").await;
        if another_window {
          d.close_window_and_go_to_handle_zero().await;
        }
      }
      
      let alice = c.alice().await;
      let token = alice.make_invoice_link().await.access_token().await?.attrs.token;
      d.goto(&format!("http://localhost:8000/#/invoice/{}", token)).await;
      check_i_am_in_buy_tokens_page(&d, false).await;
      signup_and_verify(&d, &c.site).await;
      create_wizard(&d, &c.site, 12, "testing-template").await;
      sign_wizard(&d).await;
      d.click("#wizard-buy-tokens").await;
      check_i_am_in_buy_tokens_page(&d, true).await;
      d.click("#dashboard-menu-item").await;
      d.click("#dashboard-buy-tokens").await;
      check_i_am_in_buy_tokens_page(&d, true).await;

      d.goto("http://localhost:8000/#/invoices/muchas-gracias").await;
      d.wait_for_text("#invoice-link-success > div:nth-child(2) > p:nth-child(1)", r"We have received your payment*").await;
      d.goto("http://localhost:8000/#/invoices/error-al-pagar").await;
      d.wait_for_text("#invoice-link-error > div:nth-child(2) > p:nth-child(1)", r"We could not receive your payment*").await;
    }

    integration_test!{ old_invoice_endpoints_backward_compatible (c, d)
      let alice = c.alice().await;
      d.goto("http://localhost:8000/invoices/muchas-gracias").await;
      d.wait_for_text("#invoice-link-success > div:nth-child(2) > p:nth-child(1)", r"We have received your payment*").await;
      d.goto("http://localhost:8000/invoices/error-al-pagar").await;
      d.wait_for_text("#invoice-link-error > div:nth-child(2) > p:nth-child(1)", r"We could not receive your payment*").await;
      let token = alice.make_invoice_link().await.access_token().await?.attrs.token;
      d.goto(&format!("http://localhost:8000/invoices/new?link_token={}", token)).await;
      d.wait_for_text("#invoice-link-buy > div:nth-child(1) > form > div:nth-child(1) > p:nth-child(1)", r"In order to certify your documents, you need Constata Tokens*").await;
    }

    integration_test!{ see_account_state_section_when_out_of_tokens (c, d)
      signup_and_verify(&d, &c.site).await;
      create_wizard(&d, &c.site, 12, "testing-template").await;
      sign_wizard(&d).await;
      d.click("#dashboard-menu-item").await;
      d.wait_for_text(".MuiAlert-message > div", r"There's a pending payment.*").await;
      d.click(".MuiAlert-message a").await;
      d.get_handles_and_go_to_window_one().await;
      d.wait_for("#buy-tokens-title").await;
    }

    integration_test!{ uploading_csv_in_wizard (c, d)
      signup_and_verify(&d, &c.site).await;
      let files = vec![
        ("default_certos_recipients.csv", r"Arte con plastilina*"),
        ("default_certos_recipients_special.csv", r"Arte con plastiliña,*"),
        ("certos_recipients_windows.csv", r"Arte con plastiliña,*"),
        ("certos_recipients_semicolon.csv", r"Arte con plastilina*"),
        ("certos_recipients_semicolon_special.csv", r"Arte con plastiliña;*")
      ];
      for (i, (file, motive)) in files.iter().enumerate() {
        create_template(&d, "testing-template", "DIPLOMA").await;
        let csv = format!("{}/tests/resources/{}", env::current_dir().unwrap().display(), &file);
        add_recipients_with_csv(&d, &c.site, &csv).await;
        sign_wizard(&d).await;
        d.click("a[href='#/']").await;
        d.click(&format!("#issuance-section-signed a[href='#/Issuance/{}/show']", i + 1)).await;
        d.wait_for_text("#review-entries-big > tbody > tr:nth-child(1) .column-params pre > span:nth-child(2)", r"3 de marzo de 1999*").await;
        d.wait_for_text("#review-entries-big > tbody > tr:nth-child(1) .column-params pre > span:nth-child(4)", motive).await;
        d.click("#dashboard-menu-item").await;
      }
    }

    integration_test!{ cannot_access_after_org_deletion (c, d)
      signup_and_verify(&d, &c.site).await;
      create_wizard(&d, &c.site, 2, "testing-template").await;
      c.alice().await.make_org_deletion_for(1, b"person deletion").await;

      // There's no marker that the  person has been deleted.
      // So we just wait and check that we're still in the login page.
      d.wait_for("#password").await;
    }

    integration_test!{ safe_environment (c, d)
      async fn check_open_certificate(d: &Selenium) {
        d.wait_for("#iframe-valid-certificate").await.enter_frame().await.expect("to enter frame");
        d.wait_for("#document_0 .previews .preview img").await;
        d.click("#document_0 .document-index .field-1 .link-save").await;
      }

      let mut chain = TestBlockchain::new().await;
      let alice = c.alice().await;

      alice.make_pubkey_domain_endorsement().await;
      alice.make_kyc_endorsement().await;

      let download_proof_link = set_up_download_proof_link(&alice, &mut chain).await?;
      let token = download_proof_link.token().await?;
      let story = download_proof_link.document().await?.story().await?;
      alice.make_signed_document(&story, b"other doc", None).await;

      let key = TestBlockchain::default_private_key().await.unwrap();
      let content = story.proof(Network::Regtest, &key).await?.render_html(i18n::Lang::Es).unwrap();
      let content_path = "/tmp/content.html";
      std::fs::write(&content_path, &content).unwrap();

      let urls = vec!["http://localhost:8000/safe", "http://localhost:8000/#/safe"];
      for url in urls {
        d.goto(url).await;
        d.fill_in("input[type='file']", &content_path).await;
        check_open_certificate(&d).await;
      }

      let urls = vec![
        format!("http://localhost:8000/safe/{token}/show"),
        format!("http://localhost:8000/#/safe/{token}/show")
      ];
      for url in urls {
        d.goto(&url).await;
        check_open_certificate(&d).await;
      }

      let urls = vec![
        format!("http://localhost:8000/safe/{token}"),
        format!("http://localhost:8000/#/safe/{token}")
      ];
      for url in urls {
        d.goto(&url).await;
        d.wait_for("#pending_docs_title").await;
        d.click("#safe-button-download").await;
        d.click("#safe-button-view").await;
        check_open_certificate(&d).await;
      }

      d.goto(&format!("http://localhost:8000/#/safe/{token}")).await;
      d.wait_for("#pending_docs_title").await;
      chain.simulate_stamping().await;
      d.goto(&format!("http://localhost:8000/#/safe/{token}")).await;
      d.wait_until_gone("#pending_docs_title").await;
    }


    integration_test!{ public_certificates_metadata (c, d)
      async fn check_social_media(d: &Selenium, selector: &str, domain: &str) {
        d.click(selector).await;
        d.get_handles_and_go_to_window_one().await;
        let current_url = d.driver.current_url().await.expect("to get current url");
        assert_that!(current_url.as_str().rfind(domain).is_some());
        d.close_window_and_go_to_handle_zero().await;
      }

      let mut chain = TestBlockchain::new().await;
      let alice = c.alice().await;
      let org = alice.org().await;

      let download_proof_link = set_up_download_proof_link(&alice, &mut chain).await?;
      let token = download_proof_link.token().await?;

      d.goto(&format!("http://localhost:8000/#/safe/{token}")).await;
      d.click("#safe-button-change-public-certificate-state").await;
      check_social_media(&d, "#share-on-linkedin", "www.linkedin.com").await;
      check_social_media(&d, "#share-on-twitter", "twitter.com").await;
      d.click("#copy-certificate-to-clipboard").await;
      d.wait_until_gone("[role='alert']").await;

      let title = "Certification";
      let description = "Certificate issued by #1 via Constata.eu";
      let image = "https://constata.eu/assets/images/logo.png";
      check_public_certificate(&d, &title, &description, &image).await;

      alice.make_kyc_endorsement().await;
      let new_description = format!("Certificate issued by Bruce Schneier via Constata.eu");
      check_public_certificate(&d, &title, &new_description, &image).await;

      let new_image = "www.nuevaurl.com";
      let new_description = format!("Certificate issued by Nuevo Nombre");
      org.update()
        .public_name(Some("Nuevo Nombre".to_string()))
        .logo_url(Some(new_image.to_string()))
        .save().await?;
      check_public_certificate(&d, &title, &new_description, &new_image).await;

      d.click("#safe-button-change-public-certificate-state").await;
      d.wait_until_gone("#go-to-public-certificate").await;

      d.goto(&download_proof_link.public_certificate_url()).await;
      d.wait_for_text(".container-constata > p", r"We could not find the certificate.*").await;

      d.goto(&format!("http://localhost:8000/#/safe/{token}")).await;
      d.click("#delete-download-proof-link").await;
      d.click(".ra-confirm").await;
      d.wait_for("#deleted-link").await;
    }

    integration_test!{ public_certificates_according_template_kind (c, d)
      let mut chain = TestBlockchain::new().await;
      let alice = c.alice().await;

      signup_and_verify(&d, &c.site).await;
      create_wizard(&d, &c.site, 2, "testing-template").await;
      sign_wizard(&d).await;
      chain.fund_signer_wallet();
      chain.simulate_stamping().await;
      let entry = c.site.entry().find(&1).await?;
      let doc = entry.document().await?.expect("entry's document");
      let token = alice.make_download_proof_link_from_doc(&doc, 30).await.token().await?;
      d.goto(&format!("http://localhost:8000/#/safe/{token}")).await;
      d.click("#safe-button-change-public-certificate-state").await;
      d.click("#go-to-public-certificate").await;
      d.get_handles_and_go_to_window_one().await;
      d.wait_for("#iframe-certificate").await.enter_frame().await.expect("to enter frame");
      d.wait_for("#document_0").await;
      d.close_window_and_go_to_handle_zero().await;

      let title = "Web developer Course";
      let raw_description = "issued by apps.script.testing@constata.eu via Constata.eu";
      let image = "https://constata.eu/assets/images/logo.png";
      check_public_certificate(&d, &title, &format!("Diploma {raw_description}"), &image).await;

      let template = entry.request().await?.template().await?;
      template.clone().update().kind(TemplateKind::Attendance).save().await?;
      check_public_certificate(&d, &title, &format!("Certificate of attendance {raw_description}"), &image).await;

      template.clone().update().kind(TemplateKind::Badge).save().await?;
      check_public_certificate(&d, &title, &format!("Badge {raw_description}"), &image).await;

      template.update().kind(TemplateKind::Invitation).save().await?;
      check_public_certificate(&d, &title, &format!("Invitation {raw_description}"), &image).await;
    }


    integration_test!{ use_wizard_with_badge (c, d)
      signup_and_verify(&d, &c.site).await;
      create_template(&d, "template-show", "BADGE").await;
      add_all_recipients(&d, &c.site, 3).await;
      sign_wizard(&d).await;
      d.click("a[href='#/']").await;
      d.click("#issuances-menu-item").await;
      d.click("a[href='#/Issuance/1/show']").await;
      d.click("button[aria-label='Preview']").await;
      d.get_handles_and_go_to_window_one().await;
      d.wait_for("iframe").await.enter_frame().await.expect("to enter frame");
      d.wait_for(".badge").await;
      d.close_window_and_go_to_handle_zero().await;
    }

    integration_test!{ archive_and_unarchive_template (c, d)
      signup_and_verify(&d, &c.site).await;
      create_wizard(&d, &c.site, 2, "template-show").await;
      sign_wizard(&d).await;

      d.click("a[href='#/']").await;
      d.click("#templates").await;

      archive_template_one_and_verify_was_archived(&d).await;
      unarchive_template_one_and_verify_was_unarchived(&d).await;

      d.click("#templates").await;
      d.click("a[href='#/Template/1/show']").await;
      archive_template_one_and_verify_was_archived(&d).await;
      d.click("a[href='#/Template/1/show']").await;
      unarchive_template_one_and_verify_was_unarchived(&d).await;

      create_wizard(&d, &c.site, 2, "template-not-to-show").await;
      sign_wizard(&d).await;
      d.click("a[href='#/']").await;
      d.click("#templates").await;
      let selector = format!(".datagrid-body > tr:nth-child(2) #archive-button");
      d.click(&selector).await;
      confirm_archive_template(&d).await;
      
      d.click("#templateId").await;
      d.driver
        .action_chain()
        .key_down(thirtyfour::Key::Down)
        .key_up(thirtyfour::Key::Down)
        .key_down(thirtyfour::Key::Enter)
        .key_up(thirtyfour::Key::Enter)
        .perform().await.expect("to autoselect sucessfully");

      d.wait_for("#create-issuance-container").await;
      d.wait_for_text("#create-issuance-container > div > div > div > p", r"template-show*").await;
    }

    integration_test!{ shows_usage_statistics_for_issuances (c, d)
      let mut chain = TestBlockchain::new().await;
      signup_and_verify(&d, &c.site).await;
      create_wizard(&d, &c.site, 5, "testing-template").await;
      sign_wizard(&d).await;
      chain.fund_signer_wallet();
      chain.simulate_stamping().await;

      for entry in c.site.request().find(&1).await?.entry_vec().await? {
        entry.in_signed()?.try_complete().await?;
      }
      
      d.click("a[href='#/']").await;
      check_statistic(&d, 0, 0, 5, "No", 0).await;
      open_download_proof_link_and_public_certificate(&d, &c.site, 1, 1).await;
      check_statistic(&d, 1, 1, 5, "Yes", 1).await;
      open_download_proof_link_and_public_certificate(&d, &c.site, 2, 2).await;
      check_statistic(&d, 2, 3, 4, "Yes", 2).await;
      open_download_proof_link_and_public_certificate(&d, &c.site, 3, 2).await;
      check_statistic(&d, 3, 5, 3, "Yes", 2).await;
      open_download_proof_link_and_public_certificate(&d, &c.site, 4, 4).await;
      d.goto(&format!("http://localhost:8000/#")).await;
      open_download_proof_link_and_public_certificate(&d, &c.site, 5, 1).await;
      check_statistic(&d, 5, 10, 2, "Yes", 4).await;
      check_statistic(&d, 5, 10, 1, "Yes", 1).await;
    }

    pub async fn open_download_proof_link_and_public_certificate(
      d: &Selenium,
      site: &Site,
      id: i32,
      times_to_open: i32
    ) {
      let token = site.download_proof_link().find(&id).await.expect("to find download proof link").token().await.expect("to have a token");
      d.goto(&format!("http://localhost:8000/#/safe/{token}")).await;
      d.click("#safe-button-change-public-certificate-state").await;
      for _ in 0..times_to_open {
        d.click("#go-to-public-certificate").await;
        d.get_handles_and_go_to_window_one().await;
        d.close_window_and_go_to_handle_zero().await;
      }
    }


    pub async fn check_statistic(
      d: &Selenium,
      admin_visited_count: i32,
      public_visit_count: i32,
      child: i32,
      admin_visited: &str,
      public_visit: i32,
    ) {
      d.goto(&format!("http://localhost:8000/#")).await;
      d.click("#issuances-menu-item").await;
      d.wait_for_text(".column-adminVisitCount > span", &format!(r"{admin_visited_count}/5*")).await;
      d.wait_for_text(".column-publicVisitCount > span", &format!(r"{public_visit_count}*")).await;
      d.goto(&format!("http://localhost:8000/#")).await;
      d.click("#issuances-menu-item").await;
      d.click("a[href='#/Issuance/1/show']").await;
      d.wait_for_text(".ra-field-adminVisitCount > span", &format!(r"{admin_visited_count}/5*")).await;
      d.wait_for_text(".ra-field-publicVisitCount > span", &format!(r"{public_visit_count}*")).await;
      d.wait_for_text(&format!("#review-entries-big tbody > tr:nth-child({child}) .column-statistics .params:nth-child(1) span:nth-child(2)"), &format!(r"{admin_visited}*")).await;
      d.wait_for_text(&format!("#review-entries-big tbody > tr:nth-child({child}) .column-statistics .params:nth-child(2) span:nth-child(2)"), &format!(r"{public_visit}*")).await;
    }


    async fn confirm_archive_template(d: &Selenium) {
      d.wait_for_text(".MuiDialog-container h2", r"Are you sure you want to ARCHIVE this template?*").await;
      d.click(".ra-confirm").await;
      d.wait_for("#unarchive-button").await;
      d.click("#dashboard-menu-item").await;
      d.click("a[href='#/wizard']").await;
    }

    async fn archive_template_one_and_verify_was_archived(d: &Selenium) {
      d.click("#archive-button").await;
      confirm_archive_template(d).await;
      d.not_exists("#templateId").await;
      d.click("#dashboard-menu-item").await;
      d.click("#templates").await;
    }

    async fn unarchive_template_one_and_verify_was_unarchived(d: &Selenium) {
      d.click("#unarchive-button").await;
      d.wait_for_text(".MuiDialog-container h2", r"Are you sure you want to UNARCHIVE this template?*").await;
      d.click(".ra-confirm").await;
      d.wait_for("#archive-button").await;
      d.click("#dashboard-menu-item").await;
      d.click("a[href='#/wizard']").await;
      d.wait_for("#templateId").await;
      d.click("#dashboard-menu-item").await;
    }

    async fn set_up_download_proof_link(alice: &SignerClient, chain: &mut TestBlockchain) -> Result<DownloadProofLink> {
      let story = alice.clone().add_funds().await.story_with_signed_doc(&read("document.zip"), None, "").await;
      let doc = &story.documents().await?[0];
      chain.fund_signer_wallet();
      chain.simulate_stamping().await;
      Ok(alice.make_download_proof_link_from_doc(&doc, 30).await)
    }


    async fn assert_rematch(d: &Selenium, tag: &str) {
      let source_code = d.driver.source().await.expect("the source code");
      assert!(source_code.rfind(tag).is_some(), "Could not find {}", tag);
    }
    async fn check_public_certificate(d: &Selenium, title: &str, description: &str, image: &str) {
      d.click("#go-to-public-certificate").await;
      d.get_handles_and_go_to_window_one().await;
      assert_rematch(&d, "<meta name=\"twitter:card\" content=\"summary_large_image\">").await;
      assert_rematch(&d, &format!("<meta name=\"twitter:title\" content=\"{title}\">")).await;
      assert_rematch(&d, &format!("<meta name=\"twitter:description\" content=\"{description}\">")).await;
      assert_rematch(&d, "<meta name=\"twitter:creator\" content=\"@constataEu\">").await;
      assert_rematch(&d, &format!("<meta name=\"twitter:image\" content=\"{image}\">")).await;
      
      assert_rematch(&d, "<meta property=\"og:type\" content=\"website\">").await;
      assert_rematch(&d, &format!("<meta property=\"og:title\" content=\"{title}\">")).await;
      assert_rematch(&d, &format!("<meta property=\"og:description\" content=\"{description}\">")).await;
      assert_rematch(&d, &format!("<meta property=\"og:image\" content=\"{image}\">")).await;
      assert_rematch(&d, "<meta property=\"og:site_name\" content=\"Constata.EU\">").await;
      d.close_window_and_go_to_handle_zero().await;
    }

    async fn change_email_address(d: &Selenium, new_email: &str, change_keep_private: bool) {
      d.click("#section-email-address button").await;
      d.delete_letters_and_send_new_keys("#address", 31, new_email).await;
      if change_keep_private {
        d.click(".ra-input-keepPrivate").await;
      }
      d.click("#section-email-address button").await;
      d.wait_for_text("#section-email-address .MuiTypography-body2", r"Will be shown in your issued certificates*").await;
    }

    async fn reload(d: &Selenium) {
      d.goto("http://localhost:8000/not_found").await;
      d.goto("http://localhost:8000").await;
      d.wait_until_gone("[role='alert']").await;
      d.wait_for_text("h1", "Hello again!").await; 
      d.fill_in("#password", "password").await;
      d.click("button[type='submit']").await;
      d.wait_for("#constata_dashboard").await;
    }

    async fn add_recipients_with_csv(d: &Selenium, site: &Site, csv: &str) {
      d.fill_in("input[type='file']", &csv).await;
      d.click("#continue").await;
      d.wait_for("span[role='progressbar']").await;

      site.request().create_all_received().await.expect("to successfully create the entries");
      d.wait_for_text("h2", "Review and sign").await;
    }

    async fn fill_signup_form(d: &Selenium) {
      d.goto("http://localhost:8000").await;
      d.wait_until_gone("[role='alert']").await;
      d.click("#button_create_new_signature").await;
      d.wait_for("svg[data-testid='LoginIcon']").await;
      d.click("#signup-intro-submit").await;
      d.wait_for_text("h2", "About Constata's service").await;
      d.click(".ra-input-tycAccepted label").await;
      d.click("#signup-tyc-submit").await;
      d.wait_for_text("h2", "About your privacy.").await;
      d.click(".ra-input-privacyPolicyAccepted label").await;
      d.click("#signup-privacy-submit").await;
      d.wait_for_text("h2", "Create a password").await;
      d.fill_in("#password", "password").await;
      d.click("#signup-password-submit").await;
      d.wait_for_text("h2", "Paper Backup.").await;
      d.click("#signup-words-submit").await;
      d.wait_for_text("h2", "Optionally, your email, so we can:").await;
      d.fill_in("#email", "apps.script.testing@constata.eu").await;
      d.click(".ra-input-keepPrivate label").await;
      d.click("#signup-email-submit").await;
      d.fill_in("#confirmPassword", "password").await;
      d.click("#signup-confirm-password-submit").await;
      d.wait_for_text("h2", "All Done!").await;
      d.wait_for(".grecaptcha-logo").await;
      d.click("button[type='submit']").await;
    }

    async fn signup(d: &Selenium) -> String {
      fill_signup_form(d).await;
      d.wait_for("#constata_dashboard").await;
      let path = d.check_downloads_for_file("signature.json").await;
      std::fs::write("../target/artifacts/signature.json", std::fs::read(&path).unwrap()).unwrap();
      path
    }

    async fn signup_and_verify(d: &Selenium, s: &Site) -> String {
      let path = signup(d).await;
      let email = s.email_address().select().one().await.unwrap();
      s.email_address().verify_with_token(&email.access_token().await.unwrap().unwrap()).await.unwrap();
      path
    }

    async fn add_recipient(d: &Selenium, name: &str, email: &str, id: &str, n: i32) {
      d.click("#recipients > button").await;
      d.fill_in("#name", name).await;
      d.fill_in("#email", email).await;
      d.fill_in("#recipient_identification", id).await;
      d.fill_in("#custom_text", "Prueba").await;
      if n == 0 {
        d.fill_in("#motive", "Web developer Course").await;
      }
      d.click("button[type='submit']").await;
    }

    async fn create_template(d: &Selenium, template_name: &str, template_kind: &str) {
      d.click("a[href='#/wizard']").await;
      d.click(&format!("button[value='{template_kind}']")).await;
      d.fill_in("#newName", template_name).await;
      d.fill_in("#newLogoText", "Constata.eu").await;
      d.click("button[type='submit']").await;
    }

    async fn add_all_recipients(d: &Selenium, site: &Site, recipient_count: i32) {
      for i in 0..recipient_count {
        let email = format!("probando{i}@constata.eu");
        add_recipient(&d, "Luciano Carreño", &email, "82736123", i).await;
      }

      d.click("#continue").await;
      d.wait_for("span[role='progressbar']").await;

      site.request().create_all_received().await.expect("to create sucessfully the entries");
      d.wait_for_text("h2", "Review and sign").await;
    }

    async fn create_wizard(d: &Selenium, site: &Site, recipient_count: i32, template_name: &str) {
      create_template(&d, template_name, "DIPLOMA").await;
      add_all_recipients(&d, site, recipient_count).await;
    }

    async fn sign_wizard(d: &Selenium) {
      d.wait_for_text("h2", "Review and sign").await;
      d.fill_in("#password", "password").await;
      d.click("button[type='submit']").await;
      d.wait_for("span[role='progressbar']").await;
      d.wait_until_gone("span[role='progressbar']").await;
      d.wait_for_text("#done_container_loaded h2", "Done").await;
    }

    async fn create_kyc_request_and_process_it(d: &Selenium, site: &Site, action: &str) {
      let process = if action == "accept" { true } else { false };

      d.click("a[href='#/request_verification']").await;
      d.fill_in("#keepPrivate", "true").await;
      d.wait_for_text("p", "Will be shown in your issued certificates").await;
      d.fill_in("#name", "Brune").await;
      d.fill_in("#lastName", "Schni").await;

      d.fill_in("#nationality", "Argentina").await;
      autoselect_first_option(&d).await;
      d.fill_in("#birthdate", "05-04-1998").await;
      d.fill_in("#idNumber", "A14645Z").await;
      d.fill_in("#country", "Argentina").await;
      autoselect_first_option(&d).await;
      d.fill_in("#jobTitle", "CEO").await;
      d.fill_in("#legalEntityName", "Schnin Technology").await;
      d.fill_in("#legalEntityCountry", "Argentina").await;
      autoselect_first_option(&d).await;
      d.fill_in("#legalEntityTaxId", "T-859-ID").await;
      d.fill_in("#legalEntityLinkedinId", "84033677").await;

      let evidence = format!("{}/static/id_example.jpg", env::current_dir().expect("to get current directory").display());
      d.fill_in("#evidence", &evidence).await;
      d.click(".MuiButton-containedPrimary").await;
      d.wait_for_text(".MuiAlert-message", r"We have received your request*").await;

      let kyc_request = site.kyc_request().find(&1).await.expect("to find kyc request");
      assert_eq!(kyc_request.attrs.name, Some("Brune".to_string()));
      assert_eq!(kyc_request.attrs.id_number, Some("A14645Z".to_string()));
      assert_eq!(kyc_request.attrs.country, Some("ARG".to_string()));
      assert_eq!(kyc_request.attrs.job_title, Some("CEO".to_string()));
      assert_eq!(kyc_request.attrs.legal_entity_linkedin_id, Some("84033677".to_string()));

      kyc_request.in_pending().expect("to get pending kyc request").process_update(
        KycRequestProcessForm {
          name: process,
          last_name: process,
          id_number: process,
          id_type: process,
          birthdate: process,
          nationality: process,
          country: process,
          job_title: process,
          legal_entity_name: process,
          legal_entity_country: process,
          legal_entity_registration: process,
          legal_entity_tax_id: process,
          legal_entity_linkedin_id: process,
          evidence: vec![process],
        }
      ).await.expect("to process kyc request");
    }

    async fn autoselect_first_option(d: &Selenium) {
      d.driver
        .action_chain()
        .key_down(thirtyfour::Key::Down)
        .key_up(thirtyfour::Key::Down)
        .key_down(thirtyfour::Key::Enter)
        .key_up(thirtyfour::Key::Enter)
        .perform().await.expect("to autoselect sucessfully");
    }

    async fn check_autocomplete_in_kyc(d: &Selenium) {
      d.click("a[href='#/request_verification']").await;
      d.fill_in("#email[value='apps.script.testing@constata.eu']", "ok").await;
      d.fill_in("#name[value='Brune']", "ok").await;
      d.fill_in("#lastName[value='Schni']", "ok").await;
      d.fill_in("#jobTitle[value='CEO']", "ok").await;
    }
  }
}
