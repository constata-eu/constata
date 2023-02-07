pub mod document_received;
pub use document_received::*;

pub mod document_was_stamped_for_carbon_copies;
pub use document_was_stamped_for_carbon_copies::*;

pub mod email_bot_chat_greeting;
pub use email_bot_chat_greeting::*;

pub mod email_parked_documents;
pub use email_parked_documents::*;

#[derive(Debug, Clone)]
pub struct AttachmentStruct {
  pub name: String,
  pub body: String,
  pub content_type: String,
}

