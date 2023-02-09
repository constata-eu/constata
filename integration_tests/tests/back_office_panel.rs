mod back_office_panel {
  constata_lib::describe_one! {
    use integration_tests::*;
    use std::env;

    integration_test_private!{ admin_panel_adds_person_logo_and_public_name (c, d)
      c.alice().await;
      login(&d, &c).await?;

      click_menu(&d, "Org").await;
      d.click("a[href='#/Org/1'][aria-label='Edit']").await;
      d.fill_in("#publicName", "Consti").await;
      d.fill_in("#logoUrl", "https//www.example.com").await;
      d.click("button[type='submit']").await;
      d.wait_until_gone("[role='alert']").await;
      d.wait_for_text(".ra-field-publicName > span", r"Consti*").await;
    }

    integration_test_private!{ admin_panel_create_and_edit_kyc_endorsement (c, d)
      c.alice().await;
      login(&d, &c).await?;

      click_menu(&d, "Person").await;
      click_link(&d, "Person/1/show").await;
      d.wait_for("a[aria-label='Create Kyc Endorsement']").await;
      d.click("a[aria-label='Create Kyc Endorsement']").await;
      d.wait_for_text("#personId", r"1*").await;

      let evidence = format!("{}/static/certos_template.zip", env::current_dir()?.display());
      d.fill_in("#name", "Brune").await;
      d.fill_in("#lastName", "Schni").await;
      d.fill_in("#idNumber", "A14645Z").await;
      d.fill_in("#idType", "passport").await;
      d.click("#country").await;
      d.click("li[data-value='Argentina']").await;
      d.click("#nationality").await;
      d.click("li[data-value='Argentinean']").await;
      d.fill_in("#jobTitle", "CEO").await;
      d.fill_in("#legalEntityName", "Schnin Technology").await;
      d.click("#legalEntityCountry").await;
      d.click("li[data-value='Argentina']").await;
      d.fill_in("#legalEntityRegistration", "1344-Z").await;
      d.fill_in("#legalEntityTaxId", "T-859-ID").await;
      d.fill_in("#evidenceZip", &evidence).await;
      d.click("button[type='submit']").await;
      d.wait_until_gone("[role='alert']").await;
      d.click("a[href='#/KycEndorsement/1'][aria-label='Edit']").await;
      d.wait_for_text("#id", r"1*").await;

      let evidence = format!("{}/static/certos_template_edit.zip", env::current_dir()?.display());
      d.delete_letters_and_send_new_keys("#name", 5, "Bruno").await;
      d.delete_letters_and_send_new_keys("#lastName", 6, "Schnin").await;
      d.delete_letters_and_send_new_keys("#jobTitle", 3, "CTO").await;
      d.fill_in("#birthdate", &format!("06{}2022{}04", Key::Right.to_string(), Key::Left.to_string())).await;
      d.fill_in("#evidenceZip", &evidence).await;
      d.click("button[type='submit']").await;
      d.wait_until_gone("[role='alert']").await;
      d.wait_for_text(".ra-field-name > span", r"Bruno*").await;
      d.wait_for_text(".ra-field-birthdate > span", r"Jun 4, 2022*").await;
    }

    integration_test_private!{ admin_panel_create_invoice_link (c, d)
      c.alice().await;
      login(&d, &c).await?;

      click_menu(&d, "Org").await;
      click_link(&d, "Org/1/show").await;
      d.click("a[aria-label='Create InvoiceLink']").await;
      d.wait_for_text("#orgId", r"1*").await;
      d.click("button[type='submit']").await;
      d.wait_until_gone("[role='alert']").await;
      d.wait_for_text("a", r"Click here to visit").await;
    }

    integration_test_private!{ admin_panel_create_gift (c, d)
      let bob = c.bob().await;
      bob.make_email("bob@example.com").await;

      login(&d, &c).await?;

      click_menu(&d, "Org").await;
      click_link(&d, "Org/1/show").await;
      d.click("a[aria-label='Create Gift']").await;
      
      d.wait_for_text("#orgId", r"1*").await;
      d.fill_in("#tokens", "10").await;
      d.fill_in("#reason", "testing purpose").await;
      d.click("button[type='submit']").await;
      d.wait_until_gone("[role='alert']").await;
      d.wait_for_text(".ra-field-tokens > span", r"10*").await;
    }

    integration_test_private!{ admin_panel_edit_subscription (c, d)
      c.alice().await;
      login(&d, &c).await?;

      click_menu(&d, "Org").await;
      click_link(&d, "Org/1/show").await;
      d.click("button[path='subscription']").await;
      d.click("a[href='#/Subscription/1'][aria-label='Edit']").await;
      d.wait_for_text("#id", r"1*").await;

      d.delete_letters_and_send_new_keys("#maxMonthlyGift", 2, "20").await;
      d.delete_letters_and_send_new_keys("#pricePerToken", 3, "50").await;
      d.click("button[type='submit']").await;
      d.wait_until_gone("[role='alert']").await;

      d.wait_for_text(".ra-field-maxMonthlyGift > span", r"20 tokens*").await;
      d.wait_for_text(".ra-field-pricePerToken > span", r"€ 0.5*").await;
    }

    integration_test_private!{ admin_panel_add_custom_template (c, d)
      c.alice().await;
      login(&d, &c).await?;

      click_menu(&d, "Org").await;
      click_link(&d, "Org/1/show").await;
      d.click("a[aria-label='Create Template']").await;
      d.click("#kind").await;
      d.click("li[data-value='DIPLOMA']").await;
      d.fill_in("#name", "Template Custom").await;
      d.fill_in("#customMessage", "Mensaje Custom").await;
      d.fill_in("#ogTitleOverride", "Curso de Programación").await;
      d.fill_in("#schema", &d.template_custom_schema()).await;
      let evidence = format!("{}/static/custom_template.zip", env::current_dir()?.display());
      d.fill_in("#evidence", &evidence).await;
      d.click("button[type='submit']").await;
      d.wait_until_gone("[role='alert']").await;
      d.wait_for_text(".ra-field-id > span", r"1*").await;

      let template = c.site.template().find(&1).await?;
      assert_eq!(template.attrs.name, "Template Custom".to_string());
      assert_eq!(template.attrs.custom_message, Some("Mensaje Custom".to_string()));
      assert_eq!(template.attrs.og_title_override, Some("Curso de Programación".to_string()));
    }

    integration_test_private!{ make_kyc_request_without_evidence (c, d)
      let alice = c.alice().await;
      login(&d, &c).await?;

      let kyc_request = c.site.kyc_request()
        .insert(alice.make_insert_kyc_request(Some("Satoshi".to_string())).await)
        .validate_and_save().await.unwrap();

      click_menu(&d, "KycRequest").await;
      click_and_load(&d, "a[href='#/KycRequest/1'][aria-label='Process']").await;
      click_and_load(&d, "button[aria-label='Process'] .MuiButton-startIcon").await;
      d.wait_for_text(".MuiDialog-container h2", r"You are about to PROCESS this Kyc Request*").await;
      d.click(".ra-confirm").await;
      d.wait_for(".RaLoadingIndicator-loadedIcon").await;
      d.wait_for(".ra-field-id").await;

      assert_eq!(&kyc_request.reloaded().await?.attrs.state, "processed");
    }

    integration_test_private!{ admin_panel_accept_and_reject_kyc_requests (c, d)
      let alice = c.alice().await;
      login(&d, &c).await?;

      alice.kyc_request_big().await;
      click_menu(&d, "KycRequest").await;
      d.click("a[href='#/KycRequest/1'][aria-label='Process']").await;
      d.click("button[aria-label='Select None']").await;
      d.click("button[aria-label='Process']").await;
      d.wait_for_text(".MuiDialog-container h2", r"You are about to PROCESS this Kyc Request*").await;
      d.click(".ra-confirm").await;
      d.wait_for_text(".ra-field-state span", r"Processed*").await;
      click_menu(&d, "KycEndorsement").await;
      d.wait_for_text(".RaList-main p", r"No results found*").await;
      click_menu(&d, "Story").await;
      d.wait_for_text(".RaList-main p", r"No results found*").await;

      alice.kyc_request_big().await;
      click_menu(&d, "KycRequest").await;
      d.click("a[href='#/KycRequest/2'][aria-label='Process']").await;
      d.click("button[aria-label='Select All']").await;
      d.click("button[aria-label='Process']").await;
      d.wait_for_text(".MuiDialog-container h2", r"You are about to PROCESS this Kyc Request*").await;
      d.click(".ra-confirm").await;
      d.wait_for_text(".ra-field-storyId span", r"1*").await;
      d.wait_for_text(".ra-field-name > span", r"Satoshi*").await;
      d.wait_for_text(".ra-field-lastName > span", r"Buterin*").await;
      d.wait_for_text(".ra-field-jobTitle > span", r"Programmer*").await;
      click_menu(&d, "KycRequest").await;
      d.wait_for_text(".column-state span", r"Processed*").await;
      click_menu(&d, "Story").await;
      d.wait_for_text(".column-totalDocumentsCount > span", r"4*").await;
    }

    integration_test_private!{ admin_panel_partially_accept_kyc_requests (c, d)
      let alice = c.alice().await;
      login(&d, &c).await?;

      alice.kyc_request_big().await;
      click_menu(&d, "KycRequest").await;
      d.click("a[href='#/KycRequest/1'][aria-label='Process']").await;
      d.driver.query(By::Id("bool.lastName")).first().await?.click().await?;
      d.driver.query(By::Id("bool.country")).first().await?.click().await?;
      d.driver.query(By::Id("bool.jobTitle")).first().await?.click().await?;
      let evidence = d.driver.query(By::Id("bool.evidence.1")).first().await?;
      evidence.scroll_into_view().await?;
      evidence.click().await?;
      d.driver.query(By::Id("bool.evidence.2")).first().await?.click().await?;
      d.click("button[aria-label='Process']").await;
      d.wait_for_text(".MuiDialog-container h2", r"You are about to PROCESS this Kyc Request*").await;
      d.click(".ra-confirm").await;
      d.wait_for_text(".ra-field-storyId > span", r"1*").await;
      d.wait_for_text(".ra-field-name > span", r"Satoshi*").await;
      d.wait_for_text(".ra-field-nationality > span", r"Argentinian*").await;
      d.wait_for_text(".ra-field-legalEntityName > span", r"Bitcoin*").await;
      d.wait_for_text(".ra-field-legalEntityCountry > span", r"El Salvador*").await;
      d.wait_empty_string(".ra-field-lastName > span").await;
      d.wait_empty_string(".ra-field-country > span").await;
      d.wait_empty_string(".ra-field-jobTitle > span").await;
      click_menu(&d, "KycRequest").await;
      d.wait_for_text(".column-state span", r"Processed*").await;
      click_menu(&d, "Story").await;
      d.wait_for_text(".column-totalDocumentsCount > span", r"2*").await;
    }
    
    integration_test_private!{ admin_panel_make_org_deletion (c, d)
      c.alice().await;
      login(&d, &c).await?;
      click_menu(&d, "Org").await;
      click_link(&d, "Org/1/show").await;
      d.click("a[aria-label='Request Org Deletion']").await;
      d.wait_for_text("#orgId", r"1*").await;
      let evidence = format!("{}/static/certos_template.zip", env::current_dir()?.display());
      d.click("#reason").await;
      d.click("li[data-value='UserRequest']").await;
      d.fill_in("#description", "Testing").await;
      d.fill_in("#evidence", &evidence).await;
      d.click("button[type='submit']").await;
      d.wait_until_gone("[role='alert']").await;
      d.wait_for_text("span.ra-field-reason span", r"User Request").await;
      click_menu(&d, "OrgDeletion").await;
      d.click("button[aria-label='Physical Delete']").await;
      d.click(".ra-confirm").await;
      d.wait_until_gone("button[aria-label='Physical Delete']").await;
    }

    integration_test_private!{ admin_panel_missing_tokens_and_terms_acceptance (c, d)
      let bot = c.bot().await;
      let bob = c.bob().await;
      let alice = c.alice_no_money_no_tyc().await;
      let enterprise = c.enterprise().await;
      let eve = c.eve().await.signup().await;
      let robert = c.robert().await;
      let persons = [bob, alice.clone(), enterprise, eve, robert];
      for n in 0..persons.len() {
        let counter = format!("number_{}", n);
        persons[n].stories_with_signed_docs(counter.as_bytes()).await;
      }
      alice.make_invoice().await;

      for _ in 0..20 {
        create_person_with_parked(&c, &bot).await?;
      }

      login(&d, &c).await?;

      click_menu(&d, "Person").await;
      d.click("button[aria-label='Go to next page']").await;
      d.wait_for_text(".MuiTablePagination-displayedRows", r"21-26 of 26*").await;
      wait_until_loaded(&d).await;
      d.wait_for(".column-isTermsAccepted [data-testid=true]").await;
      d.wait_for(".column-isTermsAccepted [data-testid=false]").await;
      click_menu(&d, "MissingToken").await;
      d.wait_for_text(".MuiTablePagination-displayedRows", r"1-20 of 21*").await;
      d.click("button[aria-label='Go to next page']").await;
      d.wait_for_text(".MuiTablePagination-displayedRows", r"21-21 of 21*").await;
      click_menu(&d, "TermsAcceptance").await;
      d.wait_for_text("#acceptedIsSet", r"Only not accepted*").await;
      d.wait_for_text(".MuiTablePagination-displayedRows", r"1-4 of 4*").await;
    }

    integration_test_private!{ admin_panel_check_reference_field (c, d)
      let alice = c.alice().await;
      let bob = c.bob().await;
      let robert = c.robert().await;
      let enterprise = c.enterprise().await;
      let users = vec![alice, bob, robert, enterprise];
      for i in users {
        c.make_bulletin().await;
        i.make_signed_document(&i.make_story().await, "prueba".as_bytes(), None).await;
      }
      login(&d, &c).await?;

      click_menu(&d, "Document").await;
      let resources = vec!["Org", "Story"];
      for resource in resources {
        let css = format!("a[href='#/{}/1/show']", resource);
        d.wait_for_text(&format!("{} > span", css), "1").await;
        d.click(&css).await;
        d.wait_for_text(".ra-field-id > span", "1").await;
        d.click("a[href='#/Document'] svg").await;
        d.wait_for(".RaList-content").await;
      }
    }

    integration_test_private!{ admin_panel_check_all_sortable_fields (c, d)
      let bot = c.bot().await;
      create_resources(&c, &c.alice().await, &bot, b"https://alice.com","alice@gmail.com").await?;
      create_resources(&c, &c.robert().await, &bot, b"https://robert.com","robert@gmail.com").await?;
      login(&d, &c).await?;

      let sortable_resources = vec![
        "Bulletin", "Document", "Story", "Org", "Person", "TermsAcceptance", "EmailAddress", "Pubkey",
        "Telegram", "Payment", "Invoice", "InvoiceLink", "Gift", "KycEndorsement", "KycRequest", "OrgDeletion",
        "PubkeyDomainEndorsement", "AdminUser"
      ];

      for resource in sortable_resources {
        click_menu(&d, resource).await;
        d.click_all_elements("span[aria-label='Sort']", resource).await;
      }

      let not_sortable_resources = vec!["MissingToken", "TopTen"];
      for resource in not_sortable_resources {
        click_menu(&d, resource).await;
        d.wait_for(".column-id").await;
        d.not_exists("span[aria-label='Sort']").await;
      }
    }

    async fn wait_until_loaded(d: &Selenium) {
      d.wait_for(".RaLoadingIndicator-loadedIcon").await;
    }

    async fn create_resources(c: &TestDb, signer: &SignerClient, bot: &WitnessClient, domain: &[u8], email: &str) -> Result<(), anyhow::Error> {
      c.make_bulletin().await;
      signer.make_signed_document(&signer.make_story().await, domain, None).await;
      signer.make_email(email).await;
      signer.make_telegram().await;
      signer.make_invoice().await;
      signer.make_invoice_link().await;
      signer.make_gift().await;
      signer.make_kyc_endorsement().await;
      signer.kyc_request_big().await;
      signer.make_pubkey_domain_endorsement_for_domain(domain).await;
      signer.make_org_deletion(b"deletion").await;
      c.creates_admin_user(email, "password").await;
      c.add_funds_to_all_clients().await;
      create_person_with_parked(&c, &bot).await?;

      Ok(())
    }

    async fn create_person_with_parked(c: &TestDb, bot: &WitnessClient) -> Result<(), anyhow::Error> {
      let person = c.create_enterprise_person().await;
      person.get_or_create_terms_acceptance().await?.accept(b"acceptando").await?;
      let story = person.state.story().create(person.attrs.id, None, "prueba".to_string(), i18n::Lang::Es).await?;
      bot.witnessed_email_with_person_id(&story, person.attrs.id, b"prueba", None).await;

      Ok(())
    }

    async fn login(d: &Selenium, c: &TestDb) -> Result<constata_lib::models::admin_user::AdminUser, anyhow::Error> {
      let username = "testing@constata.eu";
      let password = "password";
      let admin = c.creates_admin_user(username, password).await;

      d.goto("http://localhost:8000/admin#/login").await;
      d.fill_in("#username", username).await;
      d.fill_in("#password", password).await;
      d.fill_in("#otp", &admin.get_current_otp()?).await;
      d.click(".RaLoginForm-button").await;
      Ok(admin)
    }

    async fn click_link(d: &Selenium, path: &str) {
      click_and_load(d, &format!("a[href='#/{path}']")).await;
    }

    async fn click_menu(d: &Selenium, path: &str) {
      d.wait_for(".MuiDrawer-root").await;
      click_and_load(d, &format!("a[href='#/{path}'] svg")).await;
    }

    async fn click_and_load(d: &Selenium, selector: &str) {
      d.click(selector).await;
      d.wait_for(".RaLoadingIndicator-loadedIcon").await;
    }
  }
}
