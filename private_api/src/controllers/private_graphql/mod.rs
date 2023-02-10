use super::*;
use juniper::{FieldResult, FieldError, graphql_value, EmptySubscription, GraphQLObject, GraphQLInputObject, graphql_object};
use rust_decimal::prelude::ToPrimitive;
use constata_lib::models::{
  UtcDateTime,
  Decimal,
  invoice_link::{self, InvoiceLinkOrderBy, SelectInvoiceLink, InsertInvoiceLink},
  person::{self, PersonOrderBy, SelectPerson},
  email_address::{self, EmailAddressOrderBy, SelectEmailAddress},
  pubkey::{self, PubkeyOrderBy, SelectPubkey},
  telegram::{self, TelegramUserOrderBy, SelectTelegramUser},
  document::{self, SelectDocument, DocumentOrderBy},
  org::{self, OrgOrderBy, SelectOrg},
  story::{self, SelectStory, StoryOrderBy},
  payment::{self, PaymentOrderBy, SelectPayment},
  PaymentSource, DocumentSource,
  bulletin::{self, BulletinOrderBy, SelectBulletin},
  invoice::{self, InvoiceOrderBy, SelectInvoice},
  subscription::{self, SubscriptionOrderBy, SelectSubscription},
  gift::{self, GiftOrderBy, SelectGift},
  terms_acceptance::{self, TermsAcceptanceOrderBy, SelectTermsAcceptance},
  kyc_request::{self, KycRequestOrderBy, SelectKycRequest},
  kyc_request_evidence::{self, KycRequestEvidenceOrderBy, SelectKycRequestEvidence},
  KycRequestProcessForm,
  kyc_endorsement::{self, KycEndorsementOrderBy, SelectKycEndorsement},
  pubkey_domain_endorsement::{self, PubkeyDomainEndorsementOrderBy, SelectPubkeyDomainEndorsement},
  admin_user::{self, AdminUserOrderBy, SelectAdminUser, AdminRole, hash_pass},
  admin_user_session::AdminUserSession,
  org_deletion::{self, OrgDeletionOrderBy, SelectOrgDeletion},
  DeletionReason,
  TemplateKind,
  certos::{
    template::{self, TemplateOrderBy, SelectTemplate, InsertTemplate},
  },
};
use sqlx_models_orm::*;
use constata_lib::base64;
use juniper_rocket::{GraphQLResponse, GraphQLRequest};

pub mod admin_user_graphql;
pub mod bulletin_graphql;
pub mod document_graphql;
pub mod download_proof_link_graphql;
pub mod email_address_graphql;
pub mod gift_graphql;
pub mod invoice_graphql;
pub mod invoice_link_graphql;
pub mod kyc_endorsement_graphql;
pub mod kyc_request_graphql;
pub mod kyc_request_evidence_graphql;
pub mod missing_token_graphql;
pub mod org_deletion_graphql;
pub mod org_graphql;
pub mod payment_graphql;
pub mod person_graphql;
pub mod pubkey_domain_endorsement_graphql;
pub mod pubkey_graphql;
pub mod story_graphql;
pub mod subscription_graphql;
pub mod telegram_graphql;
pub mod terms_acceptance_graphql;
pub mod top_ten_graphql;
pub mod template_graphql;

pub use admin_user_graphql::{AdminUser, AdminUserFilter};
pub use bulletin_graphql::{Bulletin, BulletinFilter};
pub use document_graphql::{Document, DocumentFilter};
pub use download_proof_link_graphql::{DownloadProofLink};
pub use email_address_graphql::{EmailAddress, EmailAddressFilter};
pub use gift_graphql::{Gift, GiftFilter};
pub use invoice_graphql::{Invoice, InvoiceFilter};
pub use invoice_link_graphql::{InvoiceLink, InvoiceLinkFilter};
pub use kyc_endorsement_graphql::{KycEndorsement, KycEndorsementFilter, KycEndorsementInput};
pub use kyc_request_graphql::{KycRequest, KycRequestFilter};
pub use kyc_request_evidence_graphql::{KycRequestEvidence, KycRequestEvidenceFilter};
pub use missing_token_graphql::{MissingToken};
pub use org_deletion_graphql::{OrgDeletion, OrgDeletionFilter};
pub use org_graphql::{Org, OrgFilter};
pub use payment_graphql::{Payment, PaymentFilter};
pub use person_graphql::{Person, PersonFilter};
pub use pubkey_domain_endorsement_graphql::{PubkeyDomainEndorsement, PubkeyDomainEndorsementFilter};
pub use pubkey_graphql::{Pubkey, PubkeyFilter};
pub use story_graphql::{Story, StoryFilter};
pub use subscription_graphql::{Subscription, SubscriptionFilter};
pub use telegram_graphql::{Telegram, TelegramFilter};
pub use terms_acceptance_graphql::{TermsAcceptance, TermsAcceptanceFilter};
pub use top_ten_graphql::{TopTen};
pub use template_graphql::{Template, TemplateFilter};


#[rocket::get("/graphiql")]
pub fn graphiql() -> rocket::response::content::RawHtml<String> {
  juniper_rocket::graphiql_source("/graphql", None)
}

#[rocket::get("/?<request>")]
pub async fn get_graphql_handler(
  site: &State<Site>,
  request: GraphQLRequest,
  schema: &State<Schema>,
  session: AdminUserSession,
) -> GraphQLResponse {
  in_transaction(site.inner(), request, session, schema).await
}

#[rocket::post("/", data = "<request>")]
pub async fn post_graphql_handler(
  site: &State<Site>,
  request: GraphQLRequest,
  schema: &State<Schema>,
  session: AdminUserSession,
) -> GraphQLResponse {
  in_transaction(site.inner(), request, session, schema).await
}

pub async fn in_transaction(
  site: &Site,
  request: GraphQLRequest,
  session: AdminUserSession,
  schema: &Schema,
) -> GraphQLResponse {
  let err = ||{ GraphQLResponse::error(field_error("unexpected_error_in_graphql","")) };

  let tx= match site.person().transactional().await {
    Ok(s) => s,
    _ => return err(),
  };

  let site = tx.select().state;

  let response = request.execute(&*schema, &Context{
    site: site,
    role: session.role().await.unwrap(),
    id: *session.admin_user_id()
  }).await;

  if tx.commit().await.is_err() {
    return err();
  }

  response
}

pub struct Context {
  site: Site,
  role: AdminRole,
  id: i32,
}

impl juniper::Context for Context {}

const DEFAULT_PER_PAGE: i32 = 20;
const DEFAULT_PAGE: i32 = 0;

#[rocket::async_trait]
trait Showable<Model: SqlxModel<State=Site>, Filter: Send>: Sized {
  fn sort_field_to_order_by(field: &str) -> Option<<Model as SqlxModel>::ModelOrderBy>;
  fn filter_to_select(f: Filter) -> <Model as SqlxModel>::SelectModel;
  async fn db_to_graphql(d: Model) -> MyResult<Self>;

  async fn resource(context: &Context, id: <Model as SqlxModel>::Id) -> FieldResult<Self> 
    where <Model as SqlxModel>::Id: 'async_trait
  {
    let resource = <<Model as SqlxModel>::ModelHub>::from_state(context.site.clone()).find(&id).await?;
    Ok(Self::db_to_graphql(resource).await?)
  }

  async fn collection(
    context: &Context,
    page: Option<i32>,
    per_page: Option<i32>,
    sort_field: Option<String>,
    sort_order: Option<String>,
    filter: Option<Filter>
  ) -> FieldResult<Vec<Self>>
    where Filter: 'async_trait
  {
    let limit = per_page.unwrap_or(DEFAULT_PER_PAGE);
    if limit >= 500 {
      return Err(FieldError::new(
        "Invalid pagination",
        graphql_value!({ "internal_error": "Invalid pagination" })
      ));
    }
    let offset = page.unwrap_or(DEFAULT_PAGE) * limit;

    let maybe_order_by = match sort_field {
      None => None,
      Some(ref field) => {
        if let Some(order_by) = Self::sort_field_to_order_by(field) {
          Some(order_by)
        } else {
          return Err(FieldError::new("Invalid sort_field", graphql_value!({ "invalid_sort": format!("{:?}", &sort_field) })))
        }
      }
    }; 

    let selected = <Model as SqlxModel>::SelectModelHub::from_state(context.site.clone())
      .use_struct(filter.map(Self::filter_to_select).unwrap_or(Default::default()))
      .maybe_order_by(maybe_order_by)
      .limit(limit.into())
      .offset(offset.into())
      .desc(sort_order == Some("DESC".to_string()))
      .all().await?;

    let mut all = vec![];
    for p in selected.into_iter() {
      all.push(Self::db_to_graphql(p).await?);
    }
    Ok(all)
  }

  async fn count( context: &Context, filter: Option<Filter>) -> FieldResult<ListMetadata>
    where Filter: 'async_trait
  {
    let count = <Model as SqlxModel>::SelectModelHub::from_state(context.site.clone())
      .use_struct(filter.map(Self::filter_to_select).unwrap_or(Default::default()))
      .count().await?
      .to_i32()
      .ok_or(FieldError::new("too_many_records", graphql_value!({})))?;

    Ok(ListMetadata{count})
  }
}


#[derive(GraphQLObject)]
pub struct ListMetadata {
  count: i32
}


pub struct Query;

macro_rules! make_graphql_query {
  (
    $version:literal;
    showables {
      $([$resource_type:ident, $collection:ident, $meta:tt, $meta_name:literal, $filter_type:ty, $id_type:ty],)*
    }
    $($extra:tt)*
  ) => (
    #[graphql_object(context=Context)]
    impl Query {
      fn api_version() -> &'static str { $version }

      $(
        #[allow(non_snake_case)]
        async fn $resource_type(context: &Context, id: $id_type) -> FieldResult<$resource_type> {
          <$resource_type>::resource(context, id).await
        }

        #[allow(non_snake_case)]
        async fn $collection(context: &Context, page: Option<i32>, per_page: Option<i32>, sort_field: Option<String>, sort_order: Option<String>, filter: Option<$filter_type>) -> FieldResult<Vec<$resource_type>> {
          <$resource_type>::collection(context, page, per_page, sort_field, sort_order, filter).await
        }

        #[graphql(name=$meta_name)]
        #[allow(non_snake_case)]
        async fn $meta(context: &Context, _page: Option<i32>, _per_page: Option<i32>, _sort_field: Option<String>, _sort_order: Option<String>, filter: Option<$filter_type>) -> FieldResult<ListMetadata> {
          <$resource_type>::count(context, filter).await
        }
      )*

      $($extra)*
    }
  )
}

make_graphql_query!{
  "1.0";
  showables {
    [Bulletin, allBulletins, allBulletinsMeta, "_allBulletinsMeta", BulletinFilter, i32],
    [Document, allDocuments, allDocumentsMeta, "_allDocumentsMeta", DocumentFilter, String],
    [Org, allOrgs, allOrgsMeta, "_allOrgsMeta", OrgFilter, i32],
    [Person, allPeople, allPeopleMeta, "_allPeopleMeta", PersonFilter, i32],
    [EmailAddress, allEmailAddresses, allEmailAddressesMeta, "_allEmailAddressesMeta", EmailAddressFilter, i32],
    [Pubkey, allPubkeys, allPubkeysMeta, "_allPubkeysMeta", PubkeyFilter, String],
    [Telegram, allTelegrams, allTelegramsMeta, "_allTelegramsMeta", TelegramFilter, String],
    [Payment, allPayments, allPaymentsMeta, "_allPaymentsMeta", PaymentFilter, i32],
    [Invoice, allInvoices, allInvoicesMeta, "_allInvoicesMeta", InvoiceFilter, i32],
    [InvoiceLink, allInvoiceLinks, allInvoiceLinksMeta, "_allInvoiceLinksMeta", InvoiceLinkFilter, i32],
    [Subscription, allSubscriptions, allSubscriptionsMeta, "_allSubscriptionsMeta", SubscriptionFilter, i32],
    [Gift, allGifts, allGiftsMeta, "_allGiftsMeta", GiftFilter, i32],
    [TermsAcceptance, allTermsAcceptances, allTermsAcceptancesMeta, "_allTermsAcceptancesMeta", TermsAcceptanceFilter, i32],
    [KycRequest, allKycRequests, allKycRequestsMeta, "_allKycRequestsMeta", KycRequestFilter, i32],
    [KycRequestEvidence, allKycRequestEvidences, allKycRequestEvidencesMeta, "_allKycRequestEvidencesMeta", KycRequestEvidenceFilter, i32],
    [KycEndorsement, allKycEndorsements, allKycEndorsementsMeta, "_allKycEndorsementsMeta", KycEndorsementFilter, i32],
    [PubkeyDomainEndorsement, allPubkeyDomainEndorsements, allPubkeyDomainEndorsementsMeta, "_allPubkeyDomainEndorsementsMeta", PubkeyDomainEndorsementFilter, i32],
    [Story, allStories, allStoriesMeta, "_allStoriesMeta", StoryFilter, i32],
    [OrgDeletion, allOrgDeletions, allOrgDeletionsMeta, "_allOrgDeletionsMeta", OrgDeletionFilter, i32],
    [Template, allTemplates, allTemplatesMeta, "_allTemplatesMeta", TemplateFilter, i32],
  }

  #[graphql(name="DownloadProofLink")]
  async fn download_proof_link(context: &Context, id: String) -> FieldResult<DownloadProofLink> {
    DownloadProofLink::create_link(context, id).await
  }
  
  async fn all_top_tens(context: &Context, _page: Option<i32>, _per_page: Option<i32>, _sort_field: Option<String>, _sort_order: Option<String>, _filter: Option<PersonFilter>) -> FieldResult<Vec<TopTen>> {
    Ok(TopTen::get_top(context).await?)
  }
  #[graphql(name="_allTopTensMeta")]
  async fn _all_top_tens_meta(_context: &Context, _page: Option<i32>, _per_page: Option<i32>, _sort_field: Option<String>, _sort_order: Option<String>, _filter: Option<PersonFilter>) -> FieldResult<ListMetadata> {
    Ok(ListMetadata { count: 10 })
  }

  async fn all_missing_tokens(context: &Context, page: Option<i32>, per_page: Option<i32>, _sort_field: Option<String>, _sort_order: Option<String>, _filter: Option<PersonFilter>) -> FieldResult<Vec<MissingToken>> {
    Ok(MissingToken::collection(context, page, per_page).await?)
  }

  #[graphql(name="_allMissingTokensMeta")]
  async fn _all_missing_tokens_meta(context: &Context, _page: Option<i32>, _per_page: Option<i32>, _sort_field: Option<String>, _sort_order: Option<String>, _filter: Option<PersonFilter>) -> FieldResult<ListMetadata> {
    Ok(MissingToken::count(context).await?)
  }

  #[graphql(name="AdminUser")]
  async fn admin_user(context: &Context, id: i32) -> FieldResult<AdminUser> {
    if context.role != AdminRole::SuperAdmin {
      return Err(field_error("401", "you don't have permission and you tried to hack the UI"));
    }
    AdminUser::resource(context, id).await
  }

  async fn all_admin_users(context: &Context, page: Option<i32>, per_page: Option<i32>, sort_field: Option<String>, sort_order: Option<String>, filter: Option<AdminUserFilter>) -> FieldResult<Vec<AdminUser>> {
    if context.role != AdminRole::SuperAdmin {
      return Err(field_error("401", "you don't have permission and you tried to hack the UI"));
    }
    AdminUser::collection(context, page, per_page, sort_field, sort_order, filter).await
  }

  #[graphql(name="_allAdminUsersMeta")]
  async fn _all_admin_users_meta(context: &Context, _page: Option<i32>, _per_page: Option<i32>, _sort_field: Option<String>, _sort_order: Option<String>, filter: Option<AdminUserFilter>) -> FieldResult<ListMetadata> {
    AdminUser::count(context, filter).await
  }
}


pub struct Mutation;

#[graphql_object(context=Context)]
impl Mutation {
  async fn update_org(
    context: &Context, id: i32, logo_url: Option<String>, public_name: Option<String>
  ) -> FieldResult<Org> {
    Org::update_org(context, id, logo_url, public_name).await
  }

  pub async fn create_template(
    context: &Context, org_id: i32, name: String, kind: String, evidence: String, schema: Option<String>, custom_message: Option<String>, og_title_override: Option<String>
  ) -> FieldResult<Template> {
    Template::create_template(context, org_id, name, kind, evidence, schema, custom_message, og_title_override).await
  }

  pub async fn update_template(
    context: &Context, id: i32, name: String, kind: String, schema: Option<String>, custom_message: Option<String>, og_title_override: Option<String>
  ) -> FieldResult<Template> {
    Template::update_template(context, id, name, kind, schema, custom_message, og_title_override).await
  }
  
  async fn create_invoice_link(context: &Context, org_id: i32) -> FieldResult<InvoiceLink> {
    InvoiceLink::create_invoice_link(context, org_id).await
  }

  async fn create_gift(
    context: &Context, org_id: i32, tokens: i32, reason: String
  ) -> FieldResult<Gift> {
    Gift::create_gift(context, org_id, tokens, reason).await
  }

  async fn update_subscription(
    context: &Context, id: i32, max_monthly_gift: i32, price_per_token: i32, is_active: bool
  ) -> FieldResult<Subscription> {
    Subscription::update_subscription(context, id, max_monthly_gift, price_per_token, is_active).await
  }

  async fn create_kyc_endorsement(
    context: &Context, person_id: i32, input: KycEndorsementInput
  ) -> FieldResult<KycEndorsement> {
    input.process(context, person_id).await
  }

  async fn update_kyc_endorsement(
    context: &Context, person_id: i32, input: KycEndorsementInput
  ) -> FieldResult<KycEndorsement> {
    input.process(context, person_id).await
  }

  async fn update_kyc_request(context: &Context, id: i32, form: String) -> FieldResult<KycRequest> {
    KycRequest::update_kyc_request(context, id, form).await
  }

  async fn create_org_deletion(
    context: &Context, org_id: i32, reason: String, description: String, evidence: String,
  ) -> FieldResult<OrgDeletion> {
    OrgDeletion::create_org_deletion(context, org_id, reason, description, evidence).await
  }

  async fn physical_deletion(context: &Context, org_deletion_id: i32) -> FieldResult<OrgDeletion> {
    OrgDeletion::physical_deletion(context, org_deletion_id).await
  }

  async fn create_admin_user(
    context: &Context, username: String, password: String, role: String
  ) -> FieldResult<AdminUser> {
    AdminUser::create_admin_user(context, username, password, role).await
  }

  async fn update_admin_user(
    context: &Context, password: String, otp: String, new_password: String
  ) -> FieldResult<AdminUser> {
    AdminUser::update_admin_user(context, password, otp, new_password).await
  }
}

// A root schema consists of a query and a mutation.
// Request queries can be executed against a RootNode.
pub type Schema = juniper::RootNode<'static, Query, Mutation, EmptySubscription<Context>>;

pub fn new_graphql_schema() -> Schema {
  Schema::new(Query, Mutation, EmptySubscription::<Context>::new())
}

fn into_decimal(i: Option<i32>) -> Option<Decimal> {
  i.map(|t| Decimal::new(t.into(), 0))
}

fn into_like_search(i: Option<String>) -> Option<String> {
  i.map(|s| {
    ["%".to_string(), s, "%".to_string()].concat()
 })
}

fn field_error(message: &str, second_message: &str) -> FieldError {
  FieldError::new(
      message,
      graphql_value!({ "internal_error":  second_message })
    )
}

constata_lib::describe_one! {
  
  apitest!{ private_graphql_one_bulletin (db, c, client)
    let token = client.login_and_get_token("foo", "barz").await;
    let draft = db.bulletin().current_draft().await?.1;
    let bulletin_id = draft.as_inner().id();
    let body = body_for_one_search("Bulletin", &bulletin_id, "");
    let response = client.post_with_token("/graphql/", token, body).await;
    let expected_response = expected_response_for_one("Bulletin", &bulletin_id, false, "", &0);
    assert_eq!(response, expected_response);  
  }

  apitest!{ private_graphql_all_bulletins (_db, c, client)
    let token = client.login_and_get_token("foo", "barz").await;
    c.make_bulletin().await;
    c.make_bulletin().await;

    let body = body_for_all_search("allBulletins");
    let response = client.post_for_all_search("/graphql/", token.clone(), body).await;
    let number_all = response.all_bulletins.unwrap().len() as i32;
    assert!(number_all > 1);
          
    let body_meta = body_for_meta_search("_allBulletinsMeta");
    let response_meta = client.post_for_meta_search("/graphql/", token, body_meta).await;
    let number_meta = response_meta._all_bulletins_meta.unwrap().count;
    assert!(number_all <= number_meta);
  }

  apitest!{ private_graphql_one_document (_db, c, client)
    let token = client.login_and_get_token("foo", "barz").await;
    let document = c.alice().await.signed_document(b"hello_world").await;

    let body = format!("query {{ Document(id: \"{}\") {{ id, personId }} }}", document.id());
    let response = client.post_with_token("/graphql/", token, body).await;

    let expected_response = serde_json::json![{"data":
            {"Document": { "id": document.id(), "personId": 1 }}}].to_string();
    assert_eq!(response, expected_response);  
  }

  apitest!{ private_graphql_all_documents (_db, c, client)
    let token = client.login_and_get_token("foo", "barz").await;
    c.alice().await.signed_document(b"hello_world").await;
    c.bob().await.signed_document(b"hello_world").await;

    let body = format!("query {{ allDocuments(perPage: 50, page: 0) {{ id }} }}");
    let response = client.post_for_all_search("/graphql/", token.clone(), body).await;
    let number_all = response.all_documents.unwrap().len() as i32;
    assert!(number_all > 1);
    
    let body_meta = format!("query {{ _allDocumentsMeta(perPage: 50, page: 0) {{ count }} }}");
    let response_meta = client.post_for_meta_search("/graphql/", token, body_meta).await;
    let number_meta = response_meta._all_documents_meta.unwrap().count;
    assert!(number_all <= number_meta);
  }

  apitest!{ private_graphql_one_person (_db, c, client)
    let token = client.login_and_get_token("foo", "barz").await;
    c.alice().await;
    let body = body_for_one_search("Person", &1, "");
    let response = client.post_with_token("/graphql/", token, body).await;
    let expected_response = expected_response_for_one("Person", &1, false, "", &0);
    assert_eq!(response, expected_response);  
  }

  apitest!{ private_graphql_all_people (_db, c, client)
    let token = client.login_and_get_token("foo", "barz").await;
    c.alice().await;
    c.bob().await;

    let body = body_for_all_search("allPeople");
    let response = client.post_for_all_search("/graphql/", token.clone(), body).await;
    let number_all = response.all_people.unwrap().len() as i32;
    assert!(number_all > 1);
    
    let body_meta = body_for_meta_search("_allPeopleMeta");
    let response_meta = client.post_for_meta_search("/graphql/", token, body_meta).await;
    let number_meta = response_meta._all_people_meta.unwrap().count;
    assert!(number_all <= number_meta);
  }

  apitest!{ private_graphql_one_payment (_db, c, client)
    let token = client.login_and_get_token("foo", "barz").await;
    c.alice().await.fund().await;
    let body = body_for_one_search("Payment", &1, "orgId");
    let response = client.post_with_token("/graphql/", token, body).await;
    let expected_response = expected_response_for_one("Payment", &1, true, "orgId", &1);
    assert_eq!(response, expected_response);  
  }

  apitest!{ private_graphql_all_payments (_db, c, client)
    let token = client.login_and_get_token("foo", "barz").await;
    c.alice().await.fund().await;
    c.bob().await.fund().await;

    let body = body_for_all_search("allPayments");
    let response = client.post_for_all_search("/graphql/", token.clone(), body).await;
    let number_all = response.all_payments.unwrap().len() as i32;
    assert!(number_all > 1);
    
    let body_meta = body_for_meta_search("_allPaymentsMeta");
    let response_meta = client.post_for_meta_search("/graphql/", token, body_meta).await;
    let number_meta = response_meta._all_payments_meta.unwrap().count;
    assert!(number_all <= number_meta);
  }

  apitest!{ private_graphql_one_email (_db, c, client)
    let token = client.login_and_get_token("foo", "barz").await;
    let email = c.alice().await.make_email("probandoemail@gmail.com").await;
    let body = body_for_one_search("EmailAddress", &1, "personId");
    let response = client.post_with_token("/graphql/", token, body).await;
    let expected_response = expected_response_for_one("EmailAddress", email.id(), true, "personId", &1);
    assert_eq!(response, expected_response);  
  }

  apitest!{ private_graphql_all_emails (_db, c, client)
    let token = client.login_and_get_token("foo", "barz").await;
    c.alice().await.make_email("probandoemail2@gmail.com").await;
    c.bob().await.make_email("probandoemail3@gmail.com").await;
    let body = body_for_all_search("allEmailAddresses");
    let response = client.post_for_all_search("/graphql/", token.clone(), body).await;
    let number_all = response.all_email_addresses.unwrap().len() as i32;
    assert!(number_all > 1);
    
    let body_meta = body_for_meta_search("_allEmailAddressesMeta");
    let response_meta = client.post_for_meta_search("/graphql/", token, body_meta).await;
    let number_meta = response_meta._all_email_addresses_meta.unwrap().count;
    assert!(number_all <= number_meta);
  }

  apitest!{ private_graphql_one_invoice (_db, c, client)
    let token = client.login_and_get_token("foo", "barz").await;
    let invoice = c.alice().await.make_invoice().await;
    let body = body_for_one_search("Invoice", invoice.id(), "orgId");
    let response = client.post_with_token("/graphql/", token, body).await;
    let expected_response = expected_response_for_one("Invoice", invoice.id(), true, "orgId", &1);
    assert_eq!(response, expected_response);  
  }

  apitest!{ private_graphql_all_invoices (_db, c, client)
    let token = client.login_and_get_token("foo", "barz").await;
    c.alice().await.make_invoice().await;
    c.bob().await.make_invoice().await;

    let body = body_for_all_search("allInvoices");
    let response = client.post_for_all_search("/graphql/", token.clone(), body).await;
    let number_all = response.all_invoices.unwrap().len() as i32;
    assert!(number_all > 1);
    
    let body_meta = body_for_meta_search("_allInvoicesMeta");
    let response_meta = client.post_for_meta_search("/graphql/", token, body_meta).await;
    let number_meta = response_meta._all_invoices_meta.unwrap().count;
    assert!(number_all <= number_meta);
  }

  apitest!{ private_graphql_one_invoice_link (_db, c, client)
    let token = client.login_and_get_token("foo", "barz").await;
    let invoice_link = c.alice().await.make_invoice_link().await;
    let body = body_for_one_search("InvoiceLink", invoice_link.id(), "orgId");
    let response = client.post_with_token("/graphql/", token, body).await;
    let expected_response = expected_response_for_one("InvoiceLink", invoice_link.id(), true, "orgId", &1);
    assert_eq!(response, expected_response);  
  }

  apitest!{ private_graphql_all_invoice_links (_db, c, client)
    let token = client.login_and_get_token("foo", "barz").await;
    c.alice().await.make_invoice_link().await;
    c.bob().await.make_invoice_link().await;

    let body = body_for_all_search("allInvoiceLinks");
    let response = client.post_for_all_search("/graphql/", token.clone(), body).await;
    let number_all = response.all_invoice_links.unwrap().len() as i32;
    assert!(number_all > 1);
    
    let body_meta = body_for_meta_search("_allInvoiceLinksMeta");
    let response_meta = client.post_for_meta_search("/graphql/", token, body_meta).await;
    let number_meta = response_meta._all_invoice_links_meta.unwrap().count;
    assert!(number_all <= number_meta);
  }

  apitest!{ private_graphql_one_subscription (_db, c, client)
    let token = client.login_and_get_token("foo", "barz").await;
    let subscription = c.alice().await.org().await.subscription_or_err().await?;
    let body = body_for_one_search("Subscription", subscription.id(), "orgId");
    let response = client.post_with_token("/graphql/", token, body).await;
    let expected_response = expected_response_for_one("Subscription", subscription.id(), true, "orgId", &1);
    assert_eq!(response, expected_response);  
  }

  apitest!{ private_graphql_all_subscriptions (_db, c, client)
    let token = client.login_and_get_token("foo", "barz").await;
    c.alice().await;
    c.bob().await;

    let body = body_for_all_search("allSubscriptions");
    let response = client.post_for_all_search("/graphql/", token.clone(), body).await;
    let number_all = response.all_subscriptions.unwrap().len() as i32;
    assert!(number_all > 1);
    
    let body_meta = body_for_meta_search("_allSubscriptionsMeta");
    let response_meta = client.post_for_meta_search("/graphql/", token, body_meta).await;
    let number_meta = response_meta._all_subscriptions_meta.unwrap().count;
    assert!(number_all <= number_meta);
  }

  apitest!{ private_graphql_one_gift (_db, c, client)
    let token = client.login_and_get_token("foo", "barz").await;
    let gift = c.alice().await.make_gift().await;
    let body = body_for_one_search("Gift", gift.id(), "orgId");
    let response = client.post_with_token("/graphql/", token, body).await;
    let expected_response = expected_response_for_one("Gift", gift.id(), true, "orgId", &1);
    assert_eq!(response, expected_response);  
  }

  apitest!{ private_graphql_all_gifts (_db, c, client)
    let token = client.login_and_get_token("foo", "barz").await;
    c.alice().await.make_gift().await;
    c.bob().await.make_gift().await;

    let body = body_for_all_search("allGifts");
    let response = client.post_for_all_search("/graphql/", token.clone(), body).await;
    let number_all = response.all_gifts.unwrap().len() as i32;
    assert!(number_all > 1);
    
    let body_meta = body_for_meta_search("_allGiftsMeta");
    let response_meta = client.post_for_meta_search("/graphql/", token, body_meta).await;
    let number_meta = response_meta._all_gifts_meta.unwrap().count;
    assert!(number_all <= number_meta);
  }

  fn body_for_one_search(recurso: &str, id: &i32, segundo_campo: &str) -> String {
    format!("query {{ {}(id: {})
                   {{ id, {} }} }}",
                   recurso, id, segundo_campo)
  }

  fn body_for_all_search(recurso: &str) -> String {
    format!("query {{ {}(perPage: 50, page: 0)
                   {{ id }} }}", recurso)
  }

  fn body_for_meta_search(recurso: &str) -> String {
    format!("query {{ {}(perPage: 50, page: 0)
                   {{ count }} }}", recurso)
  }

  fn expected_response_for_one(recurso: &str, id: &i32, segundo_campo: bool, segundo_campo_key: &str, segundo_campo_value: &i32) -> String {
    if segundo_campo {
      return serde_json::json![{"data":
            {recurso.to_string(): {
                "id": id,
                segundo_campo_key.to_string(): segundo_campo_value,
            }}}].to_string()
    } else {
      return serde_json::json![{"data":
            {recurso.to_string(): {
                "id": id
            }}}].to_string()
    }
  }

}
