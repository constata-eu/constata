use super::*;

#[derive(Serialize, clap::Args)]
#[serde(rename_all = "camelCase")]
pub struct AttestationState {
  #[arg(help="Id of the attestation we want to check")]
  id: i32,
  #[arg(help="The state we expect the attestation to be in")]
  state: State,
  #[arg(short, long, help="Waits a reasonable time until the attestation reaches the expected state. \
    The 'reasonable time' varies depending on the desired state. \
    Going from 'parked' to 'processing' may take up to a few hours if a user action is needed, \
    but going from 'processing' to 'done' may take just over an hour. \
    If you rather not wait, we can send you web callbacks, check the command help.")]
  wait: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Serialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum State {
  Parked,
  Processing,
  UpdatesParked,
  UpdatesProcessing,
  Done,
}

impl AttestationState {
  pub fn run(&self, client: &Client) -> ClientResult<bool> {
    utils::wait_for_state(client, &self, self.state, self.wait, "Attestation", |current, expected| {
      match expected {
        State::Parked => true,
        State::Processing => current == State::Parked,
        _ => false,
      }
    })
  }
}
