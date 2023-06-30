use super::*;
use chrono::Datelike;
use i18n::Lang;

model!{
  state: Site,
  table: orgs,
  struct Org {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    subscription_id: Option<i32>,
    #[sqlx_model_hints(varchar, default)]
    stripe_customer_id: Option<String>,
    #[sqlx_model_hints(varchar, default)]
    public_name: Option<String>,
    #[sqlx_model_hints(varchar, default)]
    logo_url: Option<String>,
    #[sqlx_model_hints(boolean, default)]
    use_verifier: bool,
    #[sqlx_model_hints(int4, default)]
    deletion_id: Option<i32>,
    #[sqlx_model_hints(timestamptz, default)]
    created_at: UtcDateTime,
    #[sqlx_model_hints(varchar, default)]
    web_callbacks_url: Option<String>,
  },
  has_many {
    Person(org_id),
    Story(org_id),
    Document(org_id),
    Subscription(org_id),
    EmailAddress(org_id),
    Pubkey(org_id),
    PubkeyDomainEndorsement(org_id),
    KycEndorsement(org_id),
    Invoice(org_id),
    InvoiceLink(org_id),
    TermsAcceptance(org_id),
    KycRequest(org_id),
    ParkedReminder(org_id),
    Issuance(org_id),
    Template(org_id),
    Entry(org_id),
    Attestation(org_id),
    WebCallback(org_id),
  },
  belongs_to {
    OrgDeletion(deletion_id),
    Subscription(subscription_id),
  },
  queries {
    top_ten("id IN (
        SELECT org_id FROM payments GROUP BY org_id ORDER BY SUM(amount) DESC LIMIT 10)
        ORDER BY (SELECT SUM(amount) FROM payments WHERE org_id = orgs.id) DESC
    "),
    missing_tokens("id IN (
      SELECT d.org_id FROM documents d
      INNER JOIN terms_acceptances ta ON d.org_id = ta.org_id
      AND ta.accepted IS NOT NULL AND NOT d.funded
      GROUP BY d.org_id
    )
    ORDER BY (SELECT SUM(cost) FROM documents WHERE NOT funded AND org_id = orgs.id)
    DESC LIMIT $1 OFFSET $2
    ", limit: i64, offset: i64),
    has_two_day_old_parkeds_and_email_and_no_recent_reminders("id IN (
      SELECT d.org_id FROM documents d
      INNER JOIN email_addresses ea ON d.org_id = ea.org_id
      LEFT JOIN parked_reminders pr ON d.org_id = pr.org_id
      WHERE d.bulletin_id IS NULL
        AND NOT d.funded
        AND d.created_at < ($1::TIMESTAMPTZ - INTERVAL '2 DAYS')
        AND (pr.created_at IS NULL OR pr.created_at < ($1::TIMESTAMPTZ - INTERVAL '30 DAYS'))
      GROUP BY d.org_id)
    ", current: UtcDateTime),
  },
}

impl OrgHub {  
  pub async fn orgs_that_are_missing_tokens_count(&self) -> ConstataResult<i64> {
    let count: i64 = self.state.db.fetch_one_scalar(sqlx::query_scalar!(r#"
      SELECT COUNT(DISTINCT d.org_id)::bigint as "count!" FROM documents d
      INNER JOIN terms_acceptances ta ON d.org_id = ta.org_id
      AND ta.accepted IS NOT NULL AND NOT d.funded
    "#)).await?;

    Ok(count)
  }
}

impl Org {
  pub async fn subscription_or_err(&self) -> ConstataResult<Subscription> {
    self.subscription().await?
      .ok_or_else(|| Error::validation("organization_subscription","no_subscription"))
  }

  pub async fn admin(&self) -> sqlx::Result<Person> {
    if let Some(u) = self.person_scope().admin_eq(true).optional().await? {
      return Ok(u);
    }

    self.person_scope().one().await?.update().admin(true).save().await
  }

  pub async fn billing(&self) -> sqlx::Result<Person> {
    if let Some(u) = self.person_scope().billing_eq(true).optional().await? {
      return Ok(u);
    }

    self.person_scope().one().await?.update().billing(true).save().await
  }

  pub async fn name_for_on_behalf_of(&self) -> ConstataResult<String> {
    if self.org_deletion().await?.is_some() {
      return Ok(format!("#{}", self.id()))
    }

    if let Some(name) = self.public_name() {
      return Ok(name.clone())
    }

    if let Some(k) = self.admin().await?.kyc_endorsement().await? {
      if k.name().is_some() {
        return Ok(
          format!(
            "{} {}",
            &k.name().clone().unwrap_or_else(|| "".to_string()),
            &k.last_name().clone().unwrap_or_else(|| "".to_string())
          )
        )
      }

      if let Some(a) = k.legal_entity_name() {
        return Ok(a.clone())
      }
    }

    if let Some(a) = self.admin().await?.verified_email_address().await? {
      return Ok(a.address().clone())
    }

    Ok(format!("#{}", self.id()))
  }

  pub async fn account_state(&self) -> ConstataResult<AccountState> {
    AccountState::find_for(self.state.clone(), self.attrs.id).await
  }

  pub async fn date_first_and_last_document_parked(&self) -> (Option<UtcDateTime>, Option<UtcDateTime>) {
    let documents = self.document_scope().funded_eq(&false).all().await.ok();
    if let Some(docs) = documents {
      if docs.len() > 0 {
        let mut first_parked = docs[0].attrs.created_at;
        let mut last_parked = docs[0].attrs.created_at;
        for document in docs.into_iter() {
          if first_parked > document.attrs.created_at {
            first_parked = document.attrs.created_at
          }
          if last_parked < document.attrs.created_at {
            last_parked = document.attrs.created_at
          }
        }
        if first_parked == last_parked { return (None, Some(last_parked))};
        return (Some(first_parked), Some(last_parked));
      }
    }
    return (None, None);
  }

  pub async fn get_or_create_stripe_customer_id(self) -> ConstataResult<stripe::CustomerId> {
    use std::collections::HashMap;
    use stripe::{CreateCustomer, Customer};
   
    if let Some(id) = self.attrs.stripe_customer_id {
      return Ok(id.parse::<stripe::CustomerId>()?);
    }

    let mut metadata = HashMap::new();
    metadata.insert("org_id".to_string(), self.attrs.id.to_string());
    let customer_id = Customer::create(&self.state.stripe_client, CreateCustomer{
      email: self.admin().await?.email_address().await?.map(|i| i.attrs.address).as_deref(),
      metadata: Some(metadata), 
      ..Default::default()
    }).await?.id;

    self.update().stripe_customer_id(Some(customer_id.to_string())).save().await?;

    Ok(customer_id)
  }

  pub async fn get_or_create_invoice_link(&self) -> ConstataResult<InvoiceLink> {
    let maybe_existing = self.invoice_link_scope()
      .invoice_id_is_set(false)
      .optional().await?;

    match maybe_existing {
      Some(existing) => Ok(existing),
      None => {
        Ok(self.state.invoice_link()
          .insert(InsertInvoiceLink::from_org(&self).await?)
          .save().await?)
      }
    }
  }
  
  pub async fn get_or_create_terms_acceptance(&self) -> sqlx::Result<TermsAcceptance> {
    self.admin().await?.get_or_create_terms_acceptance().await
  }
}

impl Default for InsertOrg {
  fn default() -> Self {
    InsertOrg{ subscription_id: None }
  }
}

impl InsertOrgHub {
  pub async fn save_and_subscribe(self, lang: Lang) -> ConstataResult<Org> {
    self.save_and_subscribe_with_plan("Early Bird", Decimal::ZERO, Decimal::new(10,0), Decimal::ONE, lang).await
  }

  pub async fn save_and_subscribe_enterprise(self, lang: Lang) -> ConstataResult<Org> {
    self.save_and_subscribe_with_plan("Enterprise", Decimal::new(20,0), Decimal::ZERO, Decimal::new(5,1), lang).await
  }

  pub async fn save_and_subscribe_with_plan(mut self, name: &str, required: Decimal, gift: Decimal, price: Decimal, lang: Lang) -> ConstataResult<Org> {
    let tx = self.state.org().transactional().await?;
    self.state.db = tx.state.db.clone();
    let temp_org = self.save().await?;

    let subscription = tx.state.subscription()
      .insert(InsertSubscription{
        org_id: temp_org.attrs.id,
        created_at: Utc::now(),
        is_active: true,
        stripe_subscription_id: None,
        invoicing_day: Utc::now().day() as i32,
        plan_name: name.to_string(),
        max_monthly_gift: gift,
        required_token_purchase: required,
        price_per_token: price,
        default_payment_source: None,
      }).save().await?;

    let org = temp_org.update().subscription_id(Some(subscription.attrs.id)).save().await?;

    tx.state.person().create(&org, true, true, lang, "InsertOrgHub::save_and_subscribe").await?;

    tx.commit().await?;

    Ok(org)
  }
}

describe! {
  use rust_decimal_macros::dec;

  dbtest!{ has_top_ten_queries (site, c)
    // Alice starts out with 10 free tokens.
    // We want alice to have paid the least and have the biggest parked document.
    let alice = c.alice().await; // Alice has 10 free tokens.
    alice.signed_document(&vec![1u8; 1024 * 1024 * 10]).await;
    alice.signed_document(&vec![1u8; 1024 * 1024 * 11]).await;
    c.add_funds_to_org_with_amount(*alice.org().await.id(), dec!(5)).await;

    // Enterprise has no free tokens, we want k
    // We want enterprise to have paid the most and have the smallest parked document.
    let enterprise = c.enterprise().await;
    enterprise.signed_document(&vec![1u8; 1024 * 1024 * 9]).await;
    c.add_funds_to_org_with_amount(*enterprise.org().await.id(), dec!(8)).await;

    // Eve has the lowest funding and the smallest parked document
    let eve = c.eve().await.signup_and_accept_terms_and_conditions().await;
    eve.signed_document(&vec![1u8; 1024 * 1024 * 10]).await;
    eve.signed_document(&vec![1u8; 1024 * 1024 * 2]).await;
    c.add_funds_to_org_with_amount(*eve.org().await.id(), dec!(1)).await;

    assert_eq!(site.org().top_ten().all().await?, vec![
      enterprise.org().await,
      alice.org().await,
      eve.org().await
    ]);

    assert_eq!(site.org().missing_tokens(20, 0).all().await?, vec![
      alice.org().await,
      enterprise.org().await,
      eve.org().await
    ]);
  }
}
