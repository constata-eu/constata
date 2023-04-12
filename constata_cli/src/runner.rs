use super::*;

#[macro_export]
macro_rules! commands {
  (
    $(
      $(#[doc = $doc:expr])*
      $name:ident => $( |$runner:ident, $query:ident| $handler:block )? $( $harcoded:ident $(($message:literal))? )?,
    )*
  ) => (
    #[derive(Subcommand)]
    pub enum Commands {
      $(
        $(#[doc = $doc ])*
        $name($name),
      )*
    }

    impl Runner {
      fn dispatch(&self, command: Commands) -> ClientResult<()> {
        match command {
          $(
            Commands::$name(query) => {
              commands!(self, query, $($runner, $query, $handler)? $( $harcoded $(,$message)? )?);
            },
          )*
        }
        Ok(())
      }
    }
  );
  ($runner:ident, $query:ident, print_json ) => {
    $runner.print_json(&$query.run(&$runner.client)?)?;
  };
  ($runner:ident, $query:ident, print_json_or_save, $message:literal ) => {
    let should_print = $query.out_file.is_none();
    let result = $query.run(&$runner.client)?;
    if should_print {
      $runner.print_json(&result)?;
    } else {
      println!($message, result.id);
    }
  };
  ($runner:ident, $query:ident, $runner_bind:ident, $query_bind:ident, $($handler:tt)* ) => {
    let $runner_bind = $runner;
    let $query_bind = $query;
    $($handler)*
  };
}

pub use commands;

pub struct Runner {
  pub client: Client, 
  pub json_pointer: Option<String>,
}

impl Runner {
  pub fn run(client: Client, json_pointer: Option<String>, command: Commands) -> ClientResult<()> {
    Self{ client, json_pointer }.dispatch(command)
  }

  pub fn print_json<T: serde::Serialize>(&self, it: &T) -> ClientResult<()>{
    let value = serde_json::to_value(&it)?;
    let as_str = serde_json::to_string(&it)?;

    let json = if let Some(pointer) = &self.json_pointer {
      value.pointer(&pointer).ok_or_else(||{
        error![InvalidInput("Could not find pointer {} on response {}", &pointer, as_str)]
      })?
    } else {
      &value
    };
    
    println!("{}", serde_json::to_string_pretty(json)?);
    Ok(())
  }

  pub fn exit_with_boolean(&self, value: bool) -> ClientResult<()> {
    println!("{}", value);
    std::process::exit(if value { 0 } else { 1 });
  }
}
