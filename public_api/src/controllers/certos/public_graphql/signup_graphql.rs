use super::*;
use crate::controllers::Result as ConstataResult;
use models::{outgoing_email_message_kind::*};

#[derive(GraphQLInputObject)]
#[graphql(description = "A SignUp form")]
pub struct SignupInput {
  email: Option<String>,
  keep_private: bool,
}

#[derive(GraphQLObject)]
#[graphql(description = "A person's signup")]
pub struct Signup {
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
