use super::*;

#[derive(Serialize, clap::Args)]
#[serde(rename_all = "camelCase")]
pub struct IssuanceState {
  #[arg(help="Id of the issuance we want to check is created")]
  id: i32,
  #[arg(help="The state we expect the issuance to be in")]
  state: State,
  #[arg(short, long, help="Waits a reasonable time until issuance reaches the expected state. \
    The 'reasonable time' varies depending on the desired state.
    Going from 'received' to 'created' may take a fraction of a second, \
    but going from 'signed' to 'done' may take over an hour.\
    If you rather not wait, we can send you web callbacks, check the command help.")]
  wait: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Serialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum State {
  Received,
  Created,
  Signed,
  Completed,
  Failed,
}

impl IssuanceState {
  pub fn run(&self, client: &Client) -> ClientResult<bool> {
    utils::wait_for_state(client, &self, self.state, self.wait, "Issuance", |current, expected| {
      match expected {
        State::Received => true,
        State::Created => current != State::Received,
        State::Signed => matches!(current, State::Completed | State::Failed),
        State::Completed => current == State::Failed,
        State::Failed => matches!(current, State::Signed | State::Completed),
      }
    })
  }
}
