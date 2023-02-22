/*mod controllers;
pub use controllers::*;
pub use constata_lib::models::Decimal;
*/
fn main() {
  println!("Disabled until public_api is a lib + bin crate");
  /*
  let schema = controllers::certos::public_graphql::new_graphql_schema().as_schema_language();
  std::fs::write("public_api_schema.graphql", &schema).unwrap();
  std::fs::write("public_api_queries.graphql", graphql_queries_from_schema::generate_all(&schema).unwrap()).unwrap();
  */
}
