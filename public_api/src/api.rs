use super::*;
use bitcoin::PrivateKey;
use rust_decimal::{Decimal, prelude::ToPrimitive};
use constata_lib::{
  prelude::Error,
  graphql::{GqlScalar, Bytes},
  models::{self as db, Previewer, person::*, org::*},
};
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
use juniper_rocket::{graphiql_source, GraphQLRequest, GraphQLResponse};
use sqlx_models_orm::*;

constata_lib::pub_mods!{
  showable::{Showable, ListMetadata};
  template::{Template,TemplateInput,TemplateFilter};
  issuance::{
    Issuance,
    IssuanceFilter,
    CreateIssuanceInput,
    CreateIssuanceFromCsvInput,
    CreateIssuanceFromJsonInput,
    AppendEntriesToIssuanceInput,
    IssuanceExport
  };
  entry::{
    Entry,
    EntryFilter,
    UnsignedEntryPayload,
    SigningIteratorInput,
    EntryHtmlExport,
    PreviewEntry
  };
  account_state::{AccountState};
  kyc_request::{KycRequest, KycRequestFilter, KycRequestInput};
  attestation::{
    Attestation,
    AttestationFilter,
    AttestationInput,
    AttestationSetPublishedInput,
    AttestationHtmlExport
  };
  endorsement_manifest::{EndorsementManifest};
  vc_request::{KioskVcRequest, VcRequest, VcRequestFilter};
  vc_prompt::{VcPrompt, VcPromptFilter, CreateVcPromptInput, UpdateVcPromptInput};
  vc_requirement::{VcRequirement, VcRequirementFilter};
  email_address::{EmailAddress, EmailAddressFilter, EmailAddressInput, EmailAddressVerification};
  signup::{Signup, SignupInput};
  download_proof_link::{DownloadProofLink, DownloadProofLinkInput, AbridgedProofZip};
  invoice_link::{InvoiceLink, InvoiceLinkInput};
  web_callback::{WebCallback, WebCallbackFilter, WebCallbackAttempt, WebCallbackAttemptFilter};
  pubkey::{Pubkey, PubkeyFilter};
  proof::{Proof};
}

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
pub async fn get_handler(
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
pub async fn post_handler(
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
  let person = Person::new(site.inner().clone(), PersonAttrs{
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

  pub async fn org(&self) -> ConstataResult<Org> {
    Ok(self.current_person.person.org().await?)
  }

  pub fn person(&self) -> Person {
    self.current_person.person.clone()
  }
}

impl juniper::Context for Context {}

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
    [VcRequirement, allVcRequirements, allVcRequirementsMeta, "_allVcRequirementsMeta", VcRequirementFilter, i32],
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
      entry: Entry::db_to_graphql(entry).await?,
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
    let request = context.org().await?.issuance_scope().id_eq(&id).one().await?;
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
      attestation: api::Attestation::db_to_graphql(attestation).await?,
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
      .issuance_scope()
      .id_eq(id)
      .one().await?
      .in_created()?
      .discard().await?
      .into_inner();
    Issuance::db_to_graphql(db_request).await
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

  /*
  pub async fn update_kiosk_vc_request( context: &Context, code: String ) -> FieldResult<KioskVcRequest> {
    KioskVcRequest::update(context, &code).await
  }
  */
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
