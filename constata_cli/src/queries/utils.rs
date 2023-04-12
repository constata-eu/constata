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
macro_rules! export_verifiable_html_collection_template {
  (
    $module_name:ident,
    $query:ident,
    $collection_query:ident,
    $model:path,
    $single_export_query:ident,
    $filename_template:literal,
  ) => (
    pub mod $module_name {
      use super::*;

      #[derive(serde::Serialize, clap::Args)]
      #[serde(rename_all = "camelCase")]
      pub struct $query {
        #[arg(help="Save the verifiable HTMLs to the given directory if possible. \
          Will fail if it encounters any verifiable HTMTL is not available.")]
        pub path: PathBuf,

        #[arg(short, long, help="Do not fail if a verifiable HTML is not available yet, skip it instead.")]
        pub dont_fail_on_missing: bool,

        #[command(flatten)]
        pub subcommand: $collection_query,
      }

      impl $query {
        pub fn run<F: Fn(i32, i32, &$model)>(&self, client: &Client, before_each_save: F) -> ClientResult<i32> {
          if !self.path.is_dir() {
            return Err(Error::NotFound(format!("a directory called {}", &self.path.display())))
          }

          let output = self.subcommand.run(client)?;
          let total = output.meta.count;
          let mut current = 1;
          let mut saved = 0;

          for item in &output.all {
            before_each_save(current, total, item);

            let exported = $single_export_query {
              id: item.id, 
              out_file: Some(self.path.join(format!($filename_template, item.id))),
            }.run(client);
            current += 1;

            match exported {
              Ok(_) => saved += 1,
              Err(e) => if !self.dont_fail_on_missing { return Err(e) }
            }
          }

          Ok(saved)
        }
      }
    }
    pub use $module_name::$query;
  )
}
pub use export_verifiable_html_collection_template;

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
        #[arg(long,help="Either ASC or DESC")]
        pub sort_order: Option<String>,
      }

      #[derive(Default, serde::Serialize, clap::Args)]
      pub struct Sort {
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

