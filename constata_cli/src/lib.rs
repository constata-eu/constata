/// This library has one module for each GraphQL query or mutation available in our API.
/// Each module has its own struct representing the query parameters, and in some cases local parameters as well.
/// We wanted to make it intuitive to transition from the command line subcommands into the API.
/// All subcommands in the command line utility are straightforward representations of queries in this library.
/// For lower level, or more idiomatic access, all types are public and the Client supports arbitrary graphql queries.

pub mod error;
pub use error::{Error, ClientResult};

pub mod client;
pub use client::Client;

pub mod queries;
pub use queries::*;

pub use public_api::controllers::certos::public_graphql as gql_types;
