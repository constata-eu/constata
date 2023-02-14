use super::*;
use crate::controllers::Result as ConstataResult;
use models::{outgoing_email_message_kind::*};

#[derive(GraphQLInputObject)]
#[graphql(description = "SignupInput Object")]
pub struct SignupInput {
  #[graphql(description = "email to be registered by the person, if any")]
  email: Option<String>,
  #[graphql(description = "boolean pointing out whether the email should be registered as private or could be public")]
  keep_private: bool,
}

#[derive(GraphQLObject)]
#[graphql(description = "Signup Object")]
pub struct Signup {
  #[graphql(description = "number identifying the person who signed up")]
  id: i32,
}

impl SignupInput {
  pub async fn process(self, context: &Context) -> ConstataResult<Signup> {
    let person = context.site.person().find(context.person_id()).await?;

    person.get_or_create_terms_acceptance().await?.accept(context.current_person.evidence()).await?;
    if let Some(email) = self.email {
      let address = person.create_or_update_email_address(&email, self.keep_private).await?;
      person.state.outgoing_email_message().create(&person, &address, OutgoingEmailMessageKind::Welcome).await?;
    }

    Ok(Signup{ id: context.person_id() })
  }
}
