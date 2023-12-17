use crate::pub_mods;
use std::path::PathBuf;
use super::{Error, ClientResult, gql_types, Client, check, error};
use serde::{Deserialize, Serialize};
use constata_lib::signed_payload::SignedPayload;

mod utils;
use utils::{
  query_by_id_and_save_file_template,
  collection_query_template,
  export_verifiable_html_collection_template,
};

pub mod gql_fields;

pub_mods!{
  create_issuance_from_json::CreateIssuanceFromJson;
  create_issuance_from_csv::CreateIssuanceFromCsv;
  custom_graphql::CustomGraphql;
  append_entries_to_issuance::AppendEntriesToIssuance;
  account_state::AccountState;
  issuance_state::IssuanceState;
  attestation_state::AttestationState;
  sign_issuance::SignIssuance;
  create_attestation::CreateAttestation;
  attestation_set_published::AttestationSetPublished;
  preview_sample_from_issuance::PreviewSampleFromIssuance;
  update_web_callbacks_url::UpdateWebCallbacksUrl;
  validate_web_callback::ValidateWebCallback;
}

export_verifiable_html_collection_template! {
  all_attestations_html_export,
  AllAttestationsHtmlExport,
  AllAttestations,
  gql_types::Attestation,
  AttestationHtmlExport,
  "attestation_{}.html",
}

export_verifiable_html_collection_template! {
  all_entries_html_export,
  AllEntriesHtmlExport,
  AllEntries,
  gql_types::Entry,
  EntryHtmlExport,
  "entry_{}.html",
}

query_by_id_and_save_file_template!{
  entry_html_export,
  gql_types::EntryHtmlExport,
  EntryHtmlExport,
  "Id of the entry your want to export.",
  "\
    Write the verifiable HTML file here, you can then open it with your web browser. \
    Use --json-pointer=/html to extract the HTML and print it to stdout.
  ",
  "EntryHtmlExport",
  &format!("\
    id
    entry {{
      {}
    }}
    verifiableHtml
    __typename
  ", gql_fields::ENTRY),
  verifiable_html
}

query_by_id_and_save_file_template!{
  attestation_html_export,
  gql_types::AttestationHtmlExport,
  AttestationHtmlExport,
  "Id of the attestation your want to export.",
  "\
    Write the verifiable HTML file here, you can then open it with your web browser. \
    Use --json-pointer=/html to extract the HTML and print it to stdout.
  ",
  "AttestationHtmlExport",
  &format!("\
    id
    attestation {{
      {}
    }}
    verifiableHtml
    __typename
  ", gql_fields::ATTESTATION),
  verifiable_html
}

query_by_id_and_save_file_template!{
  unsigned_entry_payload,
  gql_types::UnsignedEntryPayload,
  UnsignedEntryPayload,
  "id of the entry whose payload you want to download.",
  "\
    Write the unsigned entry payload here, it's always a zip file. \
    Use --json-pointer=/bytes to extract the HTML and print it to stdout.
  ",
  "UnsignedEntryPayload",
  gql_fields::UNSIGNED_ENTRY_PAYLOAD,
  bytes
}

query_by_id_and_save_file_template!{
  preview_entry,
  gql_types::PreviewEntry,
  PreviewEntry,
  "id of the entry you want preview",
  "\
    Write the HTML file here, you can then open it with your web browser. \
    Use --json-pointer=/html to extract the HTML and print it to stdout.\
  ",
  "PreviewEntry",
  "\
    id
    html
    __typename
  ",
  html
}

query_by_id_and_save_file_template!{
  issuance_export,
  gql_types::IssuanceExport,
  IssuanceExport,
  "id of the issuance you want to export as CSV",
  "\
    Write the CSV file here. \
    Use --json-pointer=/csv to extract the CSV and print it to stdout instead
  ",
  "IssuanceExport",
  "\
    id
    csv
    __typename
  ",
  csv
}

collection_query_template!{
  all_issuances,
  AllIssuances,
  gql_types::Issuance,
  gql_types::IssuanceFilter,
  "IssuanceFilter",
  "allIssuances",
  "_allIssuancesMeta",
  gql_fields::ISSUANCE
}

collection_query_template!{
  all_entries,
  AllEntries,
  gql_types::Entry,
  gql_types::EntryFilter,
  "EntryFilter",
  "allEntries",
  "_allEntriesMeta",
  gql_fields::ENTRY
}

collection_query_template!{
  all_templates,
  AllTemplates,
  gql_types::Template,
  gql_types::TemplateFilter,
  "TemplateFilter",
  "allTemplates",
  "_allTemplatesMeta",
  gql_fields::TEMPLATE
}

collection_query_template!{
  all_attestations,
  AllAttestations,
  gql_types::Attestation,
  gql_types::AttestationFilter,
  "AttestationFilter",
  "allAttestations",
  "_allAttestationsMeta",
  gql_fields::ATTESTATION
}

collection_query_template!{
  all_web_callbacks,
  AllWebCallbacks,
  gql_types::WebCallback,
  gql_types::WebCallbackFilter,
  "WebCallbackFilter",
  "allWebCallbacks",
  "_allWebCallbacksMeta",
  gql_fields::WEB_CALLBACK
}
