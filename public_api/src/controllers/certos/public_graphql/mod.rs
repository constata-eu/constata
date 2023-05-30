use super::*;
use crate::controllers::{
  Result as MyResult,
  Error,
  Result as ConstataResult,
  JsonResult,
  current_person::{CurrentPerson, CurrentPersonAndJson, AuthMethod},
};
pub use bitcoin::PrivateKey;

use juniper::{
  FieldResult,
  FieldError,
  graphql_value,
  EmptySubscription,
  GraphQLObject,
  GraphQLInputObject,
  graphql_object,
  IntrospectionFormat,
};

use rust_decimal::prelude::ToPrimitive;
use constata_lib::{
  graphql::GqlScalar,
  models::{
    self,
    UtcDateTime,
    Org,
    PersonId,
    person::{self, Person},
    pubkey::{self, PubkeyOrderBy, SelectPubkey},
    account_state,
    Previewer,
    kyc_request::{KycRequestOrderBy, SelectKycRequest},
    email_address::{EmailAddressOrderBy, SelectEmailAddress},
    attestation::{self, AttestationOrderBy, SelectAttestation},
    certos::{
      entry::{self, EntryOrderBy, SelectEntry},
      request::{self, RequestOrderBy, SelectRequest},
      template::{self, TemplateOrderBy, SelectTemplate},
      template_kind::TemplateKind,
    },
  },
};
use sqlx_models_orm::*;
use juniper_rocket::{graphiql_source, GraphQLRequest, GraphQLResponse};

pub mod template_graphql;
pub mod issuance_graphql;
pub mod entry_graphql;
pub mod account_state_graphql;
pub mod endorsement_manifest_graphql;
pub mod signup_graphql;
pub mod pubkey_graphql;
pub mod kyc_request_graphql;
pub mod email_address_graphql;
pub mod invoice_link_graphql;
pub mod download_proof_link_graphql;
pub mod proof_graphql;
pub mod attestation_graphql;
pub mod web_callback_graphql;
pub mod vc_prompt_graphql;
pub mod vc_request_graphql;

pub use template_graphql::{Template, TemplateFilter, TemplateInput};
pub use issuance_graphql::{
  Issuance,
  IssuanceFilter,
  IssuanceExport,
  CreateIssuanceInput,
  CreateIssuanceFromJsonInput,
  CreateIssuanceFromCsvInput,
  AppendEntriesToIssuanceInput
};
pub use entry_graphql::{Entry, EntryFilter, SigningIteratorInput, PreviewEntry, EntryHtmlExport, UnsignedEntryPayload};
pub use account_state_graphql::AccountState;
pub use endorsement_manifest_graphql::*;
pub use signup_graphql::{SignupInput, Signup};
pub use pubkey_graphql::{Pubkey, PubkeyFilter};
pub use kyc_request_graphql::{KycRequest, KycRequestInput, KycRequestFilter};
pub use email_address_graphql::{EmailAddress, EmailAddressInput, EmailAddressFilter, EmailAddressVerification};
pub use invoice_link_graphql::{InvoiceLink, InvoiceLinkInput};
pub use download_proof_link_graphql::{DownloadProofLink, DownloadProofLinkInput, AbridgedProofZip};
pub use proof_graphql::Proof;
pub use attestation_graphql::*;
pub use web_callback_graphql::*;
pub use vc_request_graphql::*;
pub use vc_prompt_graphql::*;

#[rocket::get("/graphiql")]
pub fn graphiql() -> rocket::response::content::RawHtml<String> {
  graphiql_source("/graphql", None)
}

pub async fn in_transaction(
  site: &Site,
  key: &PrivateKey,
  request: GraphQLRequest<GqlScalar>,
  non_tx_current_person: CurrentPerson,
  schema: &Schema,
  lang: i18n::Lang,
) -> GraphQLResponse {
  let err = ||{ GraphQLResponse::error(field_error("unexpected_error_in_graphql","")) };

  let Ok(tx) = site.person().transactional().await else { return err() };

  let site = tx.select().state;

  let Ok(person) = site.person().find(non_tx_current_person.person.id()).await else { return err() };

  let current_person = CurrentPerson{ person, ..non_tx_current_person };

  let response = request.execute(&*schema, &Context{ site, key: *key, current_person, lang }).await;

  if tx.commit().await.is_err() {
    return err();
  }

  response
}

#[rocket::get("/?<request>")]
pub async fn get_graphql_handler(
  state: &State<Site>,
  key: &State<PrivateKey>,
  request: GraphQLRequest<GqlScalar>,
  current_person: CurrentPerson,
  schema: &State<Schema>,
  lang: i18n::Lang,
) -> GraphQLResponse {
  in_transaction(state.inner(), key.inner(), request, current_person, schema, lang).await
}

#[rocket::post("/", data = "<current>")]
pub async fn post_graphql_handler(
  state: &State<Site>,
  key: &State<PrivateKey>,
  current: CurrentPersonAndJson<juniper::http::GraphQLBatchRequest<GqlScalar>>,
  schema: &State<Schema>,
  lang: i18n::Lang,
) -> GraphQLResponse {
  let request = juniper_rocket::GraphQLRequest::<GqlScalar>(current.json);
  in_transaction(state.inner(), key.inner(), request, current.person, schema, lang).await
}

#[rocket::get("/introspect")]
pub async fn introspect(
  site: &State<Site>,
  key: &State<PrivateKey>,
  schema: &State<Schema>,
) -> JsonResult<juniper::Value<GqlScalar>> {
  // Just any pubkey works here, because this is for generating introspection queries only.
  let person = Person::new(site.inner().clone(), person::PersonAttrs{
    id: 0,
    org_id: 0,
    deletion_id: None,
    lang: i18n::Lang::En,
    lang_set_from: "".to_string(),
    admin: false,
    billing: false,
    suspended: false,
  });

  let current_person = CurrentPerson { person, method: AuthMethod::Forced };

  let ctx = Context{
    current_person,
    site: site.inner().clone(),
    key: key.inner().clone(),
    lang: i18n::Lang::En
  };
  let (res, _errors) = juniper::introspect(&*schema, &ctx, IntrospectionFormat::default())
    .map_err(|_| Error::validation("Invalid GraphQL schema","Invalid GraphQL schema"))?;
  Ok(Json(res))
}

pub struct Context {
  site: Site,
  key: PrivateKey,
  current_person: CurrentPerson,
  lang: i18n::Lang,
}

impl Context {
  pub fn person_id(&self) -> PersonId {
    self.current_person.person.attrs.id
  }

  pub fn org_id(&self) -> i32 {
    self.current_person.person.attrs.org_id
  }

  pub async fn org(&self) -> MyResult<Org> {
    Ok(self.current_person.person.org().await?)
  }

  pub fn person(&self) -> Person {
    self.current_person.person.clone()
  }
}

impl juniper::Context for Context {}

const DEFAULT_PER_PAGE: i32 = 20;
const DEFAULT_PAGE: i32 = 0;

#[rocket::async_trait]
trait Showable<Model: SqlxModel<State=Site>, Filter: Send>: Sized {
  fn sort_field_to_order_by(field: &str) -> Option<<Model as SqlxModel>::ModelOrderBy>;
  fn filter_to_select(org_id: i32, f: Option<Filter>) -> <Model as SqlxModel>::SelectModel;
  fn select_by_id(org_id: i32, id: <Model as SqlxModel>::Id) -> <Model as SqlxModel>::SelectModel;
  async fn db_to_graphql(d: Model, with_payload: bool) -> MyResult<Self>;

  async fn resource(context: &Context, id: <Model as SqlxModel>::Id) -> FieldResult<Self> 
    where <Model as SqlxModel>::Id: 'async_trait
  {
    let resource = <<Model as SqlxModel>::ModelHub>::from_state(context.site.clone())
      .select()
      .use_struct(Self::select_by_id(context.org_id(), id))
      .one()
      .await?;
    Ok(Self::db_to_graphql(resource, false).await?)
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
      .use_struct( Self::filter_to_select(context.org_id(), filter) )
      .maybe_order_by(maybe_order_by)
      .limit(limit.into())
      .offset(offset.into())
      .desc(sort_order == Some("DESC".to_string()))
      .all().await?;

    let mut all = vec![];
    for p in selected.into_iter() {
      all.push(Self::db_to_graphql(p, false).await?);
    }
    Ok(all)
  }

  async fn count( context: &Context, filter: Option<Filter>) -> FieldResult<ListMetadata>
    where Filter: 'async_trait
  {
    let count = <Model as SqlxModel>::SelectModelHub::from_state(context.site.clone())
      .use_struct( Self::filter_to_select(context.org_id(), filter) )
      .count().await?
      .to_i32()
      .ok_or(FieldError::new("too_many_records", graphql_value!({})))?;

    Ok(ListMetadata{count})
  }
}

#[derive(Debug, GraphQLObject, serde::Serialize, serde::Deserialize)]
pub struct ListMetadata {
  pub count: i32
}

#[derive(Debug)]
pub struct Query;

macro_rules! make_graphql_query {
  (
    $version:literal;
    showables {
      $([$resource_type:ident, $collection:ident, $meta:tt, $meta_name:literal, $filter_type:ty, $id_type:ty],)*
    }
    $($extra:tt)*
  ) => (
    #[graphql_object(context=Context, scalar = GqlScalar)]
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
    [Entry, allEntries, allEntriesMeta, "_allEntriesMeta", EntryFilter, i32],
    [Issuance, allIssuances, allIssuancesMeta, "_allIssuancesMeta", IssuanceFilter, i32],
    [Template, allTemplates, allTemplatesMeta, "_allTemplatesMeta", TemplateFilter, i32],
    [Pubkey, allPubkeys, allPubkeysMeta, "_allPubkeysMeta", PubkeyFilter, String],
    [KycRequest, allKycRequests, allKycRequestsMeta, "_allKycRequestsMeta", KycRequestFilter, i32],
    [EmailAddress, allEmailAddresses, allEmailAddressesMeta, "_allEmailAddressesMeta", EmailAddressFilter, i32],
    [Attestation, allAttestations, allAttestationsMeta, "_allAttestationsMeta", AttestationFilter, i32],
    [WebCallback, allWebCallbacks, allWebCallbacksMeta, "_allWebCallbacksMeta", WebCallbackFilter, i32],
    [WebCallbackAttempt, allWebCallbackAttempts, allWebCallbackAttemptsMeta, "_allWebCallbackAttemptsMeta", WebCallbackAttemptFilter, i32],
    [VcPrompt, allVcPrompts, allVcPromptsMeta, "_allVcPromptsMeta", VcPromptFilter, i32],
    [VcRequest, allVcRequests, allVcRequestsMeta, "_allVcRequestsMeta", VcRequestFilter, i32],
  }

  #[graphql(name="PreviewEntry")]
  async fn preview_entry(context: &Context, id: i32) -> FieldResult<PreviewEntry> {
    let entry = context.org().await?.entry_scope().id_eq(&id).one().await?;
    let html = Previewer::create(
      &entry.payload().await?,
      entry.person().await?.kyc_endorsement().await?.is_some(),
    )?.render_html(context.lang)?;
    Ok(PreviewEntry{ id, html })
  }

  #[graphql(name="UnsignedEntryPayload")]
  async fn unsigned_entry_payload(context: &Context, id: i32) -> FieldResult<UnsignedEntryPayload> {
    let entry = context.org().await?.entry_scope().id_eq(&id).one().await?;
    Ok(UnsignedEntryPayload::db_to_graphql(entry).await?)
  }

  #[graphql(name="EntryHtmlExport")]
  async fn entry_html_export(context: &Context, id: i32) -> FieldResult<EntryHtmlExport> {
    let entry = context.org().await?.entry_scope().id_eq(&id).one().await?;
    let Some(verifiable_html) = entry.html_proof(&context.key, context.lang).await? else {
      return Err(field_error("not_ready", "This entry has not even been created yet, let alone verified."))
    };
    Ok(EntryHtmlExport{
      id,
      entry: Entry::db_to_graphql(entry, false).await?,
      verifiable_html
    })
  }

  #[graphql(name="AccountState")]
  async fn account_state(context: &Context, _id: i32) -> FieldResult<AccountState> {
    AccountState::from_db(context.org().await?.account_state().await?)
  }

  #[graphql(name="EndorsementManifest")]
  async fn endorsement_manifest(context: &Context, _id: i32) -> FieldResult<EndorsementManifest> {
    EndorsementManifest::from_context(context).await
  }

  #[graphql(name="EmailAddressVerification")]
  async fn email_address_verification(_context: &Context, _id: i32) -> FieldResult<EmailAddressVerification> {
    Err(field_error("access", "nothing to verify"))
  }

  #[graphql(name="InvoiceLink")]
  async fn invoice_link(context: &Context, _id: String) -> FieldResult<InvoiceLink> {
    InvoiceLink::invoice_link(context).await
  }

  #[graphql(name="KioskVcRequest")]
  async fn kiosk_vc_request(context: &Context, id: i32) -> FieldResult<KioskVcRequest> {
    KioskVcRequest::get(context, id).await
  }

  #[graphql(name="DownloadProofLink")]
  async fn download_proof_link(context: &Context, _id: String) -> FieldResult<DownloadProofLink> {
    DownloadProofLink::download_proof_link(context).await
  }

  #[graphql(name="AbridgedProofZip")]
  async fn abridged_proof_zip(context: &Context, _id: String) -> FieldResult<AbridgedProofZip> {
    DownloadProofLink::abridged_pdfs_zip(context).await
  }

  #[graphql(name="Proof")]
  async fn proof(context: &Context, _id: String) -> FieldResult<Proof> {
    Proof::proof(context).await
  }

  #[graphql(name="IssuanceExport")]
  async fn issuance_export(context: &Context, id: i32) -> FieldResult<IssuanceExport> {
    let request = context.org().await?.request_scope().id_eq(&id).one().await?;
    let csv = request.export_csv().await?;
    Ok(IssuanceExport{ id, csv })
  }

  #[graphql(name="AttestationHtmlExport")]
  async fn attestation_html_export(context: &Context, id: i32) -> FieldResult<AttestationHtmlExport> {
    let attestation = context.org().await?.attestation_scope().id_eq(&id).one().await?;
    let verifiable_html = attestation.story().await?
      .proof(context.site.settings.network, &context.key).await?
      .render_html(context.lang)?;
    Ok(AttestationHtmlExport{
      id,
      attestation: Attestation::db_to_graphql(attestation, false).await?,
      verifiable_html
    })
  }
}

pub struct Mutation;

#[graphql_object(context=Context, scalar = GqlScalar)]
impl Mutation {
  pub async fn create_signup(context: &Context, input: SignupInput) -> ConstataResult<Signup> {
    input.process(context).await
  }

  pub async fn create_issuance_from_csv(context: &Context, input: CreateIssuanceFromCsvInput) -> FieldResult<Issuance> {
    input.process(context).await
  }

  pub async fn create_issuance_from_json(context: &Context, input: CreateIssuanceFromJsonInput) -> FieldResult<Issuance> {
    input.process(context).await
  }

  pub async fn append_entries_to_issuance(context: &Context, input: AppendEntriesToIssuanceInput) -> FieldResult<Issuance> {
    input.process(context).await
  }

  pub async fn create_attestation(context: &Context, input: AttestationInput) -> FieldResult<Attestation> {
    input.create_attestation(context).await
  }

  pub async fn attestation_set_published(context: &Context, input: AttestationSetPublishedInput)
    -> FieldResult<Attestation>
  {
    input.process(context).await
  }

  pub async fn signing_iterator(context: &Context, input: SigningIteratorInput) -> FieldResult<Option<UnsignedEntryPayload>> {
    input.sign(context).await
  }
  
  pub async fn create_kyc_request(context: &Context, input: KycRequestInput) -> FieldResult<KycRequest> {
    input.process(context).await
  }

  pub async fn create_email_address(context: &Context, input: EmailAddressInput) -> FieldResult<EmailAddress> {
    input.process(context).await
  }

  pub async fn create_invoice_link(context: &Context, input: InvoiceLinkInput) -> FieldResult<InvoiceLink> {
    input.process(context).await
  }

  pub async fn update_download_proof_link(context: &Context, input: DownloadProofLinkInput) -> FieldResult<DownloadProofLink> {
    input.update_download_proof_link(context).await
  }

  pub async fn delete_download_proof_link(context: &Context) -> FieldResult<DownloadProofLink> {
    DownloadProofLink::delete_download_proof_link(context).await
  }

  pub async fn create_email_address_verification(context: &Context) -> FieldResult<EmailAddressVerification> {
    if let AuthMethod::Token{ ref token } = context.current_person.method {
      let address = context.site.email_address().verify_with_token(&token).await?;
      Ok(EmailAddressVerification{ id: address.attrs.id })
    } else {
      Err(field_error("access", "nothing to verify"))
    }
  }

  pub async fn update_issuance(context: &Context, id: i32) -> ConstataResult<Issuance> {
    let db_request = context.current_person.person
      .request_scope()
      .id_eq(id)
      .one().await?
      .in_created()?
      .discard().await?
      .into_inner();
    Issuance::db_to_graphql(db_request, false).await
  }

  pub async fn update_template(context: &Context, input: TemplateInput) -> FieldResult<Template> {
    input.update_template(context).await
  }

  pub async fn update_web_callbacks_url(context: &Context, url: Option<String>) -> FieldResult<AccountState> {
    let mut org = context.org().await?;
    org = org.update().web_callbacks_url(url).save().await?;
    AccountState::from_db(org.account_state().await?)
  }

  pub async fn create_vc_prompt(context: &Context, input: CreateVcPromptInput)
    -> FieldResult<VcPrompt>
  {
    input.process(context).await
  }

  pub async fn update_vc_prompt(context: &Context, input: UpdateVcPromptInput)
    -> FieldResult<VcPrompt>
  {
    input.process(context).await
  }

  pub async fn create_kiosk_vc_request( context: &Context, _input: Option<i32> ) -> FieldResult<KioskVcRequest> {
    KioskVcRequest::create(context).await
  }

  pub async fn update_kiosk_vc_request( context: &Context, code: String ) -> FieldResult<KioskVcRequest> {
    KioskVcRequest::update(context, &code).await
  }
}

// A root schema consists of a query and a mutation.
// Request queries can be executed against a RootNode.
pub type Schema = juniper::RootNode<'static, Query, Mutation, EmptySubscription<Context>, GqlScalar>;

pub fn new_graphql_schema() -> Schema {
  Schema::new_with_scalar_value(Query, Mutation, EmptySubscription::<Context>::new())
}

fn into_like_search(i: Option<String>) -> Option<String> {
  i.map(|s| format!("%{s}%") )
}

fn field_error(message: &str, second_message: &str) -> FieldError {
  FieldError::new(
      message,
      graphql_value!({ "internal_error":  second_message })
    )
}
