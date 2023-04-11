use super::*;

pub fn wait_for_state<V, S, F>(
  client: &Client,
  vars: &V,
  expected: S,
  wait: bool,
  resource: &str,
  should_stop_waiting: F
) -> ClientResult<bool> 
where 
  V: Serialize,
  S: for<'a> Deserialize<'a> + PartialEq + Copy + std::fmt::Debug,
  F: Fn(S,S) -> bool,
{
  for _ in 0..(60 * 60 * 48) {
    let mut result = client.query::<serde_json::Value, V>(
      vars,
      &format!("query($id: Int!){{
        {resource}(id: $id){{ state }}
      }}")
    )?;

    let Some(current) = result.pointer_mut(&format!("/{resource}/state"))
      .and_then(|x| serde_json::from_value::<S>(x.take()).ok() ) else { return Ok(false) };

    if current == expected {
      return Ok(true);
    }

    if wait {
      if should_stop_waiting(current, expected) {
        return Ok(false);
      } else {
        std::thread::sleep(std::time::Duration::from_millis(2000));
      }
    } else {
      break;
    }
  }
  Ok(false)
}

#[macro_export]
macro_rules! query_by_id_and_save_file_template {
  (
    $module_name:ident,
    $model:path,
    $query:ident,
    $id_help:literal,
    $out_file_help:literal,
    $query_name:expr,
    $query_fields:expr,
    $saved_attr:ident
  ) => (
    pub mod $module_name { 
      use super::*;

      #[derive(serde::Serialize, clap::Args)]
      #[serde(rename_all = "camelCase")]
      pub struct $query {
        #[arg(help=$id_help)]
        pub id: i32,

        #[arg(help=$out_file_help)]
        #[serde(skip)]
        pub out_file: Option<PathBuf>,
      }

      impl $query {
        pub fn run(&self, client: &Client) -> ClientResult<$model> {
          let model: $model = client.by_id(self, $query_name, $query_fields)?;

          if let Some(path) = &self.out_file {
            ex::fs::write(path, &model.$saved_attr)?;
          }

          Ok(model)
        }
      }
    }
    pub use $module_name::$query;
  )
}
pub use query_by_id_and_save_file_template;

#[macro_export]
macro_rules! collection_query_template {
  (
    $module_name:ident,
    $query:ident,
    $model:path,
    $filter_struct:path,
    $filter:literal,
    $gql_query:literal,
    $meta:literal,
    $fields:expr
  ) => (
    pub mod $module_name {
      use super::*;

      #[derive(Default, serde::Serialize, clap::Args)]
      #[serde(rename_all = "camelCase")]
      pub struct $query {
        #[command(flatten)]
        pub filter: $filter_struct,
        #[arg(long,help="The page number to fetch")]
        pub page: Option<i32>,
        #[arg(long,help="How many pages to fetch")]
        pub per_page: Option<i32>,
        #[arg(long,help="Field to use for sorting")]
        pub sort_field: Option<String>,
        #[arg(long,help="Either asc or desc")]
        pub sort_order: Option<String>,
      }

      #[derive(Debug, serde::Deserialize, serde::Serialize)]
      pub struct Collection {
        #[serde(rename=$gql_query)]
        pub all: Vec<$model>,
        #[serde(rename=$meta)]
        pub meta: gql_types::ListMetadata,
      }

      impl $query {
        pub fn run(&self, client: &Client) -> ClientResult<Collection> {
          client.query(self, &format![
            "query($page: Int, $perPage: Int, $sortField: String, $sortOrder: String, $filter: {}) {{
              {}(page: $page, perPage: $perPage, sortField: $sortField, sortOrder: $sortOrder, filter: $filter) {{
                {}
              }}
              {}(page: $page, perPage: $perPage, sortField: $sortField, sortOrder: $sortOrder, filter: $filter) {{
                count
              }}
            }}",
            $filter,
            $gql_query,
            $fields,
            $meta,
          ])
        }
      }
    }
    pub use $module_name::$query;
  )
}
pub use collection_query_template;

