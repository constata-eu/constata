use super::*;
use gql_types::entry_graphql::{EntryFilter, SigningIteratorInput, UnsignedEntryPayload};

#[derive(serde::Serialize)]
pub struct Iter<'a> {
  pub input: SigningIteratorInput,
  #[serde(skip)]
  pub client: &'a Client,
  #[serde(skip)]
  pub current: i32,
  #[serde(skip)]
  pub total: i32,
}

impl<'a> Iter<'a> {
  pub fn new(client: &'a Client, issuance_id: i32) -> ClientResult<Self> {

    let total = AllEntries{
      filter: EntryFilter{ issuance_id_eq: Some(issuance_id), ..Default::default() },
      ..Default::default()
    }.run(&client)?.meta.count;

    let current = AllEntries {
      filter: EntryFilter{
        issuance_id_eq: Some(issuance_id),
        state_eq: Some("signed".to_string()),
        ..Default::default()
      },
      ..Default::default()
    }.run(&client)?.meta.count + 1;

    Ok(Self{
      client,
      input: SigningIteratorInput { issuance_id, entry_id: None, signature: None },
      current,
      total
    })
  }

  pub fn next(&mut self) -> ClientResult<bool> {
    #[derive(Debug, serde::Deserialize)]
    struct Output {
      #[serde(rename="signingIterator")]
      pub inner: Option<UnsignedEntryPayload>,
    }

    let maybe_next = self.client.query::<Output, Self>(
      self,
      r#"mutation ($input: SigningIteratorInput!) {
        signingIterator(input: $input) {
          id
          entry {
            id
            issuanceId
            issuanceName
            rowNumber
            state
            receivedAt
            params
            errors
            documentId
            storyId
            adminVisited
            publicVisitCount
            hasEmailCallback
            emailCallbackSentAt
            payload
            adminAccessUrl
            __typename
          }
          bytes
          __typename
        }
      }"#
    ).map(|x| x.inner )?;

    if let Some(next) = maybe_next {
      self.input.entry_id = Some(next.id);
      self.input.signature = Some(self.client.sign(&next.bytes).signature.to_base64());
      Ok(true)
    } else {
      Ok(false)
    }
  }

  pub fn sign_all<F: Fn(&Self)>(&mut self, before_each: F) -> ClientResult<i32> {
    loop {
      if self.next()? {
        before_each(&self);
      } else {
        break;
      }
      self.current += 1;
    }
    Ok(self.total)
  }
}

#[derive(clap::Args)]
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignIssuance {
  #[arg(help="id of the issuance whose entries you want to sign")]
  pub id: i32,
  #[arg(short, long, help="Do not output progress information to stdout")]
  #[serde(skip)]
  pub silent: bool,
}

impl SignIssuance {
  pub fn run<F: Fn(&Iter)>(self, client: &Client, before_each: F) -> ClientResult<i32> {
    Iter::new(client, self.id)?.sign_all(before_each)
  }
}
