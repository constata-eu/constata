mod utils;
use utils::{query_by_id_and_save_file_template, collection_query_template};

use std::path::PathBuf;
use super::{Error, ClientResult, gql_types, Client, check, error};
use serde::{Deserialize, Serialize};
use constata_lib::signed_payload::SignedPayload;

pub mod gql_fields;

pub mod create_issuance_from_json;
pub use create_issuance_from_json::CreateIssuanceFromJson;

pub mod create_issuance_from_csv;
pub use create_issuance_from_csv::CreateIssuanceFromCsv;

pub mod custom_graphql;
pub use custom_graphql::CustomGraphql;

pub mod append_entries_to_issuance;
pub use append_entries_to_issuance::AppendEntriesToIssuance;

pub mod account_state;
pub use account_state::AccountState;

pub mod issuance_state;
pub use issuance_state::IssuanceState;

pub mod attestation_state;
pub use attestation_state::AttestationState;

/************************/
/* Review modules below */
/************************/

pub mod all_entries_html_export;
pub use all_entries_html_export::AllEntriesHtmlExport;

pub mod all_attestations_html_export;
pub use all_attestations_html_export::AllAttestationsHtmlExport;

pub mod sign_issuance;
pub use sign_issuance::SignIssuance;

pub mod create_attestation;
pub use create_attestation::CreateAttestation;

pub mod preview_sample_from_issuance;
pub use preview_sample_from_issuance::PreviewSampleFromIssuance;

query_by_id_and_save_file_template!{
  entry_html_export,
  gql_types::entry_graphql::EntryHtmlExport,
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
  gql_types::attestation_graphql::AttestationHtmlExport,
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
  gql_types::entry_graphql::UnsignedEntryPayload,
  UnsignedEntryPayload,
  "id of the entry whose payload you want to download.",
  "\
    Write the unsigned entry payload here, it's always a zip file. \
    Use --json-pointer=/bytes to extract the HTML and print it to stdout.
  ",
  "UnsignedEntryPayload",
  &format!("\
    id
    entry {{
      {}
    }}
    bytes
    __typename
  ", gql_fields::ENTRY),
  bytes
}

query_by_id_and_save_file_template!{
  preview_entry,
  gql_types::entry_graphql::PreviewEntry,
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
  gql_types::issuance_graphql::IssuanceExport,
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
  gql_types::issuance_graphql::Issuance,
  gql_types::issuance_graphql::IssuanceFilter,
  "IssuanceFilter",
  "allIssuances",
  "_allIssuancesMeta",
  gql_fields::ISSUANCE
}

collection_query_template!{
  all_entries,
  AllEntries,
  gql_types::entry_graphql::Entry,
  gql_types::entry_graphql::EntryFilter,
  "EntryFilter",
  "allEntries",
  "_allEntriesMeta",
  gql_fields::ENTRY
}

collection_query_template!{
  all_templates,
  AllTemplates,
  gql_types::template_graphql::Template,
  gql_types::template_graphql::TemplateFilter,
  "TemplateFilter",
  "allTemplates",
  "_allTemplatesMeta",
  gql_fields::TEMPLATE
}

collection_query_template!{
  all_attestations,
  AllAttestations,
  gql_types::attestation_graphql::Attestation,
  gql_types::attestation_graphql::AttestationFilter,
  "AttestationFilter",
  "allAttestations",
  "_allAttestationsMeta",
  gql_fields::ATTESTATION
}
