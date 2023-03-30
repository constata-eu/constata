pub mod entry_done;
pub use entry_done::*;

pub mod email_parked_documents;
pub use email_parked_documents::*;

#[derive(Debug, Clone)]
pub struct AttachmentStruct {
  pub name: String,
  pub body: String,
  pub content_type: String,
}

