use super::read;
use chrono::Utc;
use crate::{
  models::{
    *,
    person::*,
    payment::*,
    document::Accepted,
    story::*,
    bulletin::*,
    kyc_endorsement::*,
    invoice::*,
    PaymentSource,
    admin_user::{AdminUser, AdminRole},
    invoice_link::*,
    kyc_request::*,
    download_proof_link::InsertDownloadProofLink,
    TemplateSchemaField,
  },
  Result,
  signed_payload::SignedPayload,
};
use bitcoin::{secp256k1, util::misc::MessageSignature, Address, Network, PrivateKey};
use sqlx::types::Decimal;
use std::process::Command;
use std::str::FromStr;
use rust_decimal_macros::dec;
use chrono::TimeZone;
use i18n::Lang;

#[derive(Clone)]
pub struct TestDb{
  pub site: Site,
}

impl TestDb {
  pub async fn new() -> std::result::Result<Self, anyhow::Error> {
    let database_uri = std::env::var("ROCKET_DATABASE_URI").unwrap_or_else(|_| {
      "postgres://constata:password@localhost/constata_development".to_string()
    });

    let output = Command::new("sqlx")
      .current_dir("../constata_lib/db")
      .env("DATABASE_URL", &database_uri)
      .args(&["database", "reset", "-y"])
      .output()
      .expect("SQLX not available.");

    if !output.status.success() {
      // the -y option fails unless the script detects it's running in a terminal.
      // And for whatever reason, it detects a terminal in macos but not on linux.
      let two = Command::new("sqlx")
        .current_dir("../constata_lib/db")
        .args(&["-D", &database_uri, "database", "reset"])
        .output()
        .expect("SQLX not available.");
      assert!(two.status.success(), "Could not reset the database!");
    }

    Command::new("bash")
      .args(&["-c", "rm -rf /tmp/constata-local-*"])
      .output()
      .expect("Could not delete local storage of files.");

    Ok(Self{ site: Site::default_with_keyring("password").await? })
  }

  pub async fn make_bulletin(&self) -> Bulletin {
    self.site.bulletin().insert(InsertBulletin{
      state: format!("proposed"),
      started_at: Utc::now(),
    }).save().await.unwrap()
  }

  pub async fn make_person(&self) -> Person {
    self.site.org()
      .insert(Default::default())
      .save_and_subscribe(Lang::Es).await.unwrap()
      .admin().await.unwrap()
  }

  pub async fn create_enterprise_person(&self) -> Person {
    self.site.org()
      .insert(Default::default())
      .save_and_subscribe_enterprise(Lang::Es).await.unwrap()
      .admin().await.unwrap()
  }

  pub async fn eve(&self) -> SignerClient {
    // Eve has an Compressed Key and is not signed up to constata.
    SignerClient::create(self, "5JCQpY9zc1A3hQckaE1qskXwRyCFtWnqeDG4NaYrZUEsVFH6ZEU")
  }

  pub async fn robert(&self) -> SignerClient {
    SignerClient::create(
      self,
      "5JCQpY9zc1A3hQckaE1qskXwRyCFtWnqeDG4NaYrZUEsVFH6ZEU",
    )
    .signup()
    .await
  }

  pub async fn alice(&self) -> SignerClient {
    SignerClient::create(
      self,
      "cVt4o7BGAig1UXywgGSmARhxMdzP5qvQsxKkSsc1XEkw3tDTQFpy",
    )
    .signup_and_accept_terms_and_conditions()
    .await
  }

  pub async fn alice_no_money_no_tyc(&self) -> SignerClient {
    let person = self.create_enterprise_person().await;

    SignerClient::create(self, "cVt4o7BGAig1UXywgGSmARhxMdzP5qvQsxKkSsc1XEkw3tDTQFpy")
      .signup_person(person).await
  }

  pub async fn bob(&self) -> SignerClient {
    // Bob has a Compressed Key.
    SignerClient::create(self, "5JYkZjmN7PVMjJUfJWfRFwtuXTGB439XV6faajeHPAM9Z2PT2R3")
      .signup_and_accept_terms_and_conditions()
      .await
  }

  pub async fn enterprise(&self) -> SignerClient {
    let person = self.create_enterprise_person().await;

    SignerClient::create(self, "5JYkZjmN7PVMjJUfJWfRFwtuXTGB439XV6faajeHPAM9Z2PT2R3")
      .signup_person(person).await
      .accept_terms_and_conditions().await
  }

  pub async fn accept_terms_to_all_clients(&self) {
    let persons = self.site.person().select().all().await.unwrap();
    for p in persons {
      let tyc = p.get_or_create_terms_acceptance().await.unwrap();
      tyc.accept(b"").await.unwrap();
    }
  }

  pub async fn add_funds_to_all_clients(&self) {
    let orgs = self.site.org().select().all().await.unwrap();
    for o in orgs { self.add_funds_to_org(o.attrs.id).await; }
  }

  pub async fn add_funds_to_org(&self, org_id: i32) {
    self.add_funds_to_org_with_amount(org_id, Decimal::new(99,0)).await;
  }

  pub async fn add_funds_to_org_with_amount(&self, org_id: i32, amount: Decimal) {
    self.site.payment().insert(InsertPayment{
      org_id,
      created_at: Utc::now(),
      amount: amount.clone(),
      tokens: amount.clone(),
      fees: Decimal::new(1,0),
      payment_source: PaymentSource::BtcPay,
      clearing_data: "Test payment".into(),
      invoice_id: None,
    }).save_and_trigger_updates().await.unwrap();
  }

  pub async fn creates_admin_user(&self, username: &str, password: &str) -> AdminUser {
    self.site.admin_user()
      .create(username, password, AdminRole::SuperAdmin)
      .await
      .unwrap()
  }

  pub async fn creates_admin_user_for_development(&self) {
    let admin = self.site.admin_user()
      .create("development@constata.eu", "password", AdminRole::SuperAdmin)
      .await.unwrap();
    admin.update().otp_seed("GIVZS767V2UXZINNPJCBKQTO6JPPCT4N".to_string())
      .save().await.unwrap();
  }
}

#[derive(Clone)]
pub struct SignerClient {
  pub key: PrivateKey,
  pub db: TestDb,
  pub person_id: Option<PersonId>,
}

impl SignerClient {
  pub fn create(db: &TestDb, key: &str) -> SignerClient {
    Self {
      key: PrivateKey::from_str(key).unwrap(),
      db: db.clone(),
      person_id: None,
    }
  }

  pub async fn signup_and_accept_terms_and_conditions(self) -> Self {
    self.signup().await.accept_terms_and_conditions().await
  }

  pub async fn signup(self) -> Self {
    self.creates_admin_user_for_development().await;
    let person = self.db.make_person().await;
    self.signup_person(person).await
  }

  pub async fn signup_person(self, person: Person) -> Self {
    let (encrypted_key, public_key) = self.get_client_signature_keys();
    self.db.site.pubkey().create_from_signed_payload_with_backup(
      person.attrs.id,
      &self.signed_payload(b"Hello Constata.eu"),
      &encrypted_key,
      &public_key,
    ).await.unwrap();
    Self{ person_id: Some(person.attrs.id), ..self }
  }

  pub fn write_signature_json_artifact(&self) -> &'static str {
    let path = "target/artifacts/signature.json";
    let (encrypted_key, public_key) = self.get_client_signature_keys();

    std::fs::write(&format!("../{path}"),
      serde_json::to_string(&serde_json::json![{
        "public_key": public_key,
        "encrypted_key": encrypted_key,
        "environment": "development"
      }]).unwrap(),
    ).unwrap();

    path
  }

  pub fn get_client_signature_keys(&self) -> (String, String) {
    let encrypted_key = simplestcrypt::encrypt_and_serialize(b"password", self.key.to_wif().as_bytes()).unwrap();
    let public_key = self.key.public_key(&secp256k1::Secp256k1::new()).to_string();

    (hex::encode(&encrypted_key), public_key)
  }

  pub async fn accept_terms_and_conditions(self) -> Self {
    let person_id = self.person_id.expect("Need to signup this signer client for testing first");
    let person = self.db.site.person().find(&person_id).await.unwrap();
    let tyc = person.get_or_create_terms_acceptance().await.unwrap();
    tyc.accept(b"").await.unwrap();
    self
  }
  
  pub async fn creates_admin_user_for_development(&self) {
    let admin = self.db.site.admin_user()
      .create("development@constata.eu", "password", AdminRole::SuperAdmin)
      .await.unwrap();
    admin.update().otp_seed("GIVZS767V2UXZINNPJCBKQTO6JPPCT4N".to_string())
      .save().await.unwrap();
  }

  pub async fn add_funds(self) -> Self {
    self.fund().await;
    self
  }

  pub async fn fund(&self) {
    self.db.add_funds_to_org(*self.org().await.id()).await;
  }

  pub async fn make_story(&self) -> Story {
    self.make_story_with_args(None, "", Lang::Es).await
  }

  pub async fn make_story_with_args(&self, open_until: Option<UtcDateTime>, markers: &str, lang: Lang) -> Story {
    self.db.site.story().create(self.person_id(), open_until, markers.to_string(), lang).await.unwrap()
  }

  pub async fn make_signed_document(&self, story: &Story, message: &[u8], filename: Option<&str>) -> Document {
    self.db.site.document().create_from_signed_payload(
      story,
      &self.signed_payload(message),
      filename,
    ).await.unwrap()
  }

  pub async fn story_with_signed_doc(&self, message: &[u8], open_until: Option<UtcDateTime>, markers: &str) -> Story {
    let story = self.make_story_with_args(open_until, markers, i18n::Lang::Es).await;
    self.make_signed_document(&story, message, None).await;
    story
  }

  pub async fn stories_with_signed_docs(&self, prefix: &[u8]) -> Vec<Story> {
    let mut stories = vec![];
    for x in &[b"foo", b"bar", b"baz"] {
      stories.push(self.story_with_signed_doc(&[prefix, &x[..]].concat(), None, "").await);
    }
    stories
  }

  pub async fn signed_document(&self, message: &[u8]) -> Document {
    self.make_signed_document(&self.make_story().await, message, None).await
  }

  pub async fn signed_document_with_filename(&self, message: &[u8], filename: Option<&str>) -> Document {
    self.make_signed_document(&self.make_story().await, message, filename).await
  }

  pub async fn accepted_document(&self, message: &[u8]) -> Accepted {
    self.signed_document(message).await.in_accepted().expect("accepted")
  }

  pub async fn accepted_document_with_filename(&self, message: &[u8], filename: Option<&str>) -> Accepted {
    self.signed_document_with_filename(message, filename).await.in_accepted().expect("accepted")
  }

  pub async fn signed_documents(&self, prefix: &[u8]) -> Vec<Document> {
    let mut docs = vec![];
    for x in &[b"foo", b"bar", b"baz"] {
      docs.push(self.signed_document(&[prefix, &x[..]].concat()).await);
    }
    docs
  }

  pub fn signed_payload(&self, msg: &[u8]) -> SignedPayload {
    SignedPayload {
      payload: msg.to_vec(),
      signer: self.public_key(),
      signature: self.sign_message(msg),
    }
  }

  pub fn wrong_signed_payload(&self, msg: &[u8]) -> SignedPayload {
    let payload = SignedPayload {
      payload: msg.to_vec(),
      signer: self.public_key(),
      signature: self.sign_message(&b"wrong"[..]),
    };
    assert!(!payload.signed_ok().unwrap());
    payload
  }

  pub fn public_key(&self) -> Address {
    Address::p2pkh(
      &self.key.public_key(&secp256k1::Secp256k1::new()),
      Network::Regtest,
    )
  }

  pub fn sign_message(&self, message: &[u8]) -> MessageSignature {
    SignedPayload::sign_with_key(message, &self.key)
  }

  pub async fn sign_request_entry(&self, entry: Entry) -> EntrySignature {
    EntrySignature {
      entry_id: entry.attrs.id,
      signature: SignedPayload::sign_with_key(&entry.storage_fetch().await.unwrap(), &self.key),
    }
  }

  pub async fn person(&self) -> Person {
    self.db.site.person().find(&self.person_id()).await.unwrap()
  }
  
  pub async fn org(&self) -> Org {
    self.person().await.org().await.unwrap()
  }

  pub fn person_id(&self) -> PersonId {
   self.person_id.expect("Need to signup this signer client for testing first")
  }

  pub async fn make_email(&self, address: &str) -> crate::models::email_address::EmailAddress {
    self.db.site.email_address()
      .create(self.person().await, address, b"an email we received by them".to_vec(), false, false).await
      .expect("Email address to be saved")
  }

  pub async fn verify_email(&self, address: &str) -> crate::models::email_address::EmailAddress {
    self.db.site.email_address()
      .create(self.person().await, address, b"an email we received by them".to_vec(), true, false).await
      .expect("Email address to be saved")
  }

  pub async fn make_pubkey_domain_endorsement(&self) -> crate::models::PubkeyDomainEndorsement {
    self.make_pubkey_domain_endorsement_for_domain(b"https://example.com").await
  }

  pub async fn make_pubkey_domain_endorsement_for_domain(&self, domain: &[u8]) -> crate::models::PubkeyDomainEndorsement {
    let payload = self.signed_payload(domain);
    let pending = PubkeyDomainEndorsementForm{ signed_payload: payload }.save(&self.db.site).await.unwrap().into_inner();
    pending.update().state("accepted".to_string()).save().await.unwrap()
  }

  pub async fn make_org_deletion(&self, evidence: &[u8]) -> crate::models::org_deletion::OrgDeletion {
    self.make_org_deletion_for(self.org().await.attrs.id, evidence).await
  }

  pub async fn make_org_deletion_for(&self, org_id: i32, evidence: &[u8]) -> crate::models::org_deletion::OrgDeletion {
    let admin = self.db.site.admin_user().create("foo", "barz", AdminRole::Admin).await.unwrap();
    self.db.site.org_deletion().delete_org(
      org_id,
      admin.attrs.id,
      DeletionReason::UserRequest,
      "testing".to_string(),
      vec![evidence],
    ).await.unwrap()
  }

  pub async fn make_kyc_endorsement(&self) -> KycEndorsement {
    let story = self.story_with_signed_doc(b"Hello fren", None, "").await;

    self.db.site.kyc_endorsement().insert(InsertKycEndorsement{
      person_id: self.person_id(),
      org_id: self.org().await.attrs.id,
      story_id: story.attrs.id,
      name: Some("Bruce".to_string()),
      last_name: Some("Schneier".to_string()),
      id_number: Some("A12345Z".to_string()),
      id_type: Some("passport".to_string()),
      birthdate: Some(Utc.with_ymd_and_hms(1980, 1, 1, 0, 0, 0).unwrap()),
      nationality: Some("Estadounidense".to_string()),
      country: Some("España".to_string()),
      job_title: Some("CEO".to_string()),
      legal_entity_name: Some("Schneier Security".to_string()),
      legal_entity_country: Some("Chilena".to_string()),
      legal_entity_registration: Some("1234-Z".to_string()),
      legal_entity_tax_id: Some("T-789-ID".to_string()),
      legal_entity_linkedin_id: Some("84033677".to_string()),
    }).save().await.unwrap()
  }
  
  pub fn kyc_request_evidence_form(&self) -> Vec<(&str, &[u8])> {
    vec![
      ("frente_dni.txt", b"frente_dni"),
      ("dorso_dni.txt", b"dorso_dni"),
      ("selfie.txt", b"selfie"),
    ]
  }

  pub async fn make_insert_kyc_request(&self, last_name: Option<String>) -> InsertKycRequest {
    InsertKycRequest {
      person_id: self.person_id.unwrap(),
      org_id: self.org().await.attrs.id,
      name: Some("Satoshi".to_string()),
      last_name,
      id_number: Some("5442343".to_string()),
      id_type: Some("passport".to_string()),
      birthdate: Some(Utc.with_ymd_and_hms(1975, 5, 8, 0, 0, 0).unwrap()),
      nationality: Some("Argentinian".to_string()),
      country: Some("Argentina".to_string()),
      job_title: Some("Programmer".to_string()),
      legal_entity_name: Some("Bitcoin".to_string()),
      legal_entity_country: Some("El Salvador".to_string()),
      legal_entity_registration: Some("2342423".to_string()),
      legal_entity_tax_id: Some("1247565".to_string()),
      legal_entity_linkedin_id: Some("84033677".to_string()),
    }
  }

  pub async fn make_kyc_request(&self, evidence: Vec<(&str, &[u8])>, last_name: Option<String>) -> KycRequest {
    let kyc_request = self.db.site.kyc_request()
      .insert(self.make_insert_kyc_request(last_name).await)
      .validate_and_save().await.unwrap();

    for (filename, bytes) in evidence {
      self.db.site.kyc_request_evidence().create(&kyc_request, &filename, &bytes).await.unwrap();
    }

    kyc_request
  }

  pub async fn kyc_request_big(&self) -> KycRequest {
    self.make_kyc_request(self.kyc_request_evidence_form(), Some("Buterin".to_string())).await
  }

  pub async fn kyc_request_little(&self) -> KycRequest {
    self.make_kyc_request(vec![("testing.txt", b"new_selfie")], Some("Nakamoto".to_string())).await
  }

  pub async fn kyc_request_process_form(&self, boolean_1: bool, boolean_2: bool, evidence: Vec<bool>) -> KycRequestProcessForm {
    KycRequestProcessForm {
      name: boolean_1,
      last_name: boolean_2,
      id_number: boolean_1,
      id_type: boolean_1,
      birthdate: boolean_2,
      nationality: boolean_2,
      country: boolean_2,
      job_title: boolean_1,
      legal_entity_name: boolean_1,
      legal_entity_country: boolean_1,
      legal_entity_registration: boolean_1,
      legal_entity_tax_id: boolean_1,
      legal_entity_linkedin_id: boolean_1,
      evidence,
    }
  }

  pub async fn make_invoice(&self) -> Invoice {
    self.make_invoice_with_payment_source(PaymentSource::Stripe).await
  }

  pub async fn make_invoice_with_payment_source(&self, payment_source: PaymentSource) -> Invoice {
    self.db.site.invoice().insert(InsertInvoice {
      org_id: self.person().await.attrs.org_id,
      created_at: Utc::now(),
      payment_source,
      external_id: format!("probando"),
      amount: dec!(2),
      tokens: dec!(2),
      description: format!("ok"),
      url: format!("probando"),
      paid: false,
      expired: false,
      payment_id: None,
      notified_on: None,
    }).save().await.unwrap()
  }

  pub async fn make_invoice_link(&self) -> InvoiceLink {
    self.db.site.invoice_link()
      .insert(InsertInvoiceLink::from_org(&self.org().await).await.expect("to create sucessfully an invoice link"))
      .save().await.unwrap()
  }

  pub async fn make_gift(&self) -> crate::models::gift::Gift {
    self.db.site.gift().give_and_trigger_updates(
      self.person_id.unwrap(),
      Decimal::new(10, 0),
      "probando")
      .await.unwrap()
  }

  pub async fn make_download_proof_link_from_doc(&self, doc: &Document, valid_until: i64) -> crate::models::download_proof_link::DownloadProofLink {
    self.db.site.download_proof_link()
      .insert(InsertDownloadProofLink::new(doc, valid_until).await.expect("to insert a download proof link"))
      .save().await.expect("to create a download proof link")
  }

  pub async fn get_certos_id(&self) -> i32 {
    self.org().await
      .get_or_create_certos_app()
      .await.unwrap()
      .attrs.id
  }

  pub async fn make_template(&self, template_file: Vec<u8>) -> crate::models::certos::Template {
    let schema = serde_json::to_string(&vec![
      TemplateSchemaField::new("name", true, false),
      TemplateSchemaField::new("course", false, true),
      TemplateSchemaField::new("date", false, true),
      TemplateSchemaField::new("email", false, false),
    ]).unwrap();
    self.try_make_template(template_file, &schema).await.unwrap()
  }

  pub async fn try_make_template(&self, template_file: Vec<u8>, schema: &str) -> Result<crate::models::certos::Template> {
    self.db.site.template()
      .insert(InsertTemplate{
        app_id: self.get_certos_id().await,
        person_id: self.person_id.unwrap(),
        org_id: *self.org().await.id(),
        kind: TemplateKind::Diploma,
        name: "Test Template".to_string(),
        size_in_bytes: template_file.len() as i32,
        schema: schema.to_string(),
        og_title_override: None,
        custom_message: Some("Hola {{ name }} te adjuntamos este novedoso certificado".to_string()),
      }).validate_and_save(&template_file).await
  }
  
  pub async fn make_request(&self, template_id: i32, request_file: Vec<u8>) -> Result<Request> {
    Wizard{
      person: self.person().await,
      name: "certos_request.csv".to_string(),
      template: WizardTemplate::Existing {
        template_id,
      },
      csv: request_file,
    }.process().await
  }

  pub async fn make_signed_diplomas_issuance(&self) -> Result<Request> {
    let mut issuance = Wizard{
      person: self.person().await,
      name: "default_certos_recipients.csv".to_string(),
      template: WizardTemplate::New {
        kind: TemplateKind::Diploma,
        logo: ImageOrText::Text("sample diploma".to_string()),
        name: "new template".to_string(),
      },
      csv: read("default_certos_recipients.csv"),
    }.process().await?;
    self.db.site.request().create_all_received().await?;

    issuance.reload().await?;
    let created = issuance.in_created()?;
    let mut signature = None;
    while let Some(next) = created.signing_iterator(signature).await? {
      signature = Some(self.sign_request_entry(next).await);
    }
    issuance.reload().await?;
    Ok(issuance)
  }

  pub async fn make_entry_and_sign_it(&self) -> crate::models::certos::entry::Entry {
    let template = self.make_template(read("certos_template.zip")).await;
    let request = self.make_request(*template.id(), read("certos_request.csv")).await.expect("to create request");
    self.db.site.request().create_all_received().await.expect("to create all entries");

    let mut signature = None;
    let created = request.reloaded().await.expect("to reload request").in_created().expect("to get created requet");
    while let Some(next) = created.signing_iterator(signature).await.expect("to iterate sign") {
      signature = Some(self.sign_request_entry(next).await);
    }

    request.entry_vec().await.expect("to get entry vec")[1].clone()
  }
}
