use crate::{
  models::{
    model,
    hasher::hexdigest,
    PersonId,
    Org,
    OrgDeletion,
    UtcDateTime,
    DocumentSource,
    document::{self, DocumentOrderBy},
    story_snapshot::*,
    document::*,
    Site,
    Proof,
  },
  Result,
  Error,
};
use bitcoin::{ PrivateKey, network::constants::Network };
use i18n::Lang;

model!{
  state: Site,
  table: stories,
  struct Story {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    org_id: PersonId,
    #[sqlx_model_hints(timestamptz)]
    open_until: Option<UtcDateTime>,
    #[sqlx_model_hints(text)]
    markers: String,
    #[sqlx_model_hints(varchar)]
    private_markers: String,
    #[sqlx_model_hints(int4, default)]
    deletion_id: Option<i32>,
    #[sqlx_model_hints(language)]
    lang: Lang,
  },
  has_many {
    StorySnapshot(story_id),
    Document(story_id),
  },
  belongs_to {
    Org(org_id),
    OrgDeletion(deletion_id),
  }
}

impl StoryHub {
  pub async fn create(&self, org_id: i32, open_until: Option<UtcDateTime>, markers: String, lang: Lang) -> sqlx::Result<Story> {
    self.insert(InsertStory{
      org_id,
      markers: markers,
      open_until: open_until,
      private_markers: String::new(),
      lang
    }).save().await
  }

  pub async fn create_for_email_thread(&self, org_id: i32, thread_id: &str, lang: Lang) -> sqlx::Result<Story> {
    self.insert(InsertStory{
      org_id,
      markers: String::new(),
      open_until: None,
      private_markers: format!("gmail_thread:{thread_id}"),
      lang
    }).save().await
  }
}

impl Story {
  pub async fn proof<'b>(&self, network: Network, key: &'b PrivateKey) -> Result<Proof<'b>> {
    Proof::new(&self, network, key).await
  }

  pub async fn hash(&self) -> Result<String> {
    let docs: String = self.documents().await?.into_iter().map(|d| d.attrs.id ).collect();

    let preimage = format!("{}-{}-{}-{}",
      self.attrs.id,
      self.org().await?.attrs.id,
      serde_json::to_string(self.open_until())?,
      docs
    );

    Ok(hexdigest(preimage.as_bytes()))
  }

  pub async fn get_or_create_snapshot(&self) -> Result<StorySnapshot> {
    let hash = self.hash().await?;

    match self.state.story_snapshot().select().hash_eq(&hash).optional().await? {
      Some(x) => Ok(x),
      _ => {
        Ok(self.state.story_snapshot()
          .insert(InsertStorySnapshot{ story_id: self.attrs.id, hash })
          .save().await?)
      }
    }
  }

  pub async fn story_snapshots(&self) -> sqlx::Result<Vec<StorySnapshot>> {
    self.story_snapshot_scope()
      .order_by(StorySnapshotOrderBy::CreatedAt)
      .all().await
  }

  pub async fn create_download_proof_link(&self, duration_days: i64) -> Result<Option<String>> {
    let maybe_document = self.document_scope().order_by(DocumentOrderBy::CreatedAt).one().await;

    if let Ok(document) = maybe_document {
      if let Ok(accepted) = document.in_accepted() {
        if accepted.bulletin().await?.is_published() {
          return Ok(Some(document.get_or_create_download_proof_link(duration_days).await?.safe_env_url().await?))
        }
      }
    }
    
    Ok(None)
  }

  pub async fn documents(&self) -> sqlx::Result<Vec<Document>> {
    self.document_scope()
      .order_by(DocumentOrderBy::CreatedAt)
      .all().await
  }

  pub async fn published_documents(&self) -> Result<Vec<document::Accepted>> {
    let mut published = vec![];
    for doc in self.documents().await?.into_iter() {
      if let Ok(accepted) = doc.in_accepted() {
        if accepted.bulletin().await?.is_published() {
          published.push(accepted);
        }
      }
    }

    Ok(published)
  }
  
  pub async fn pending_docs(&self) -> sqlx::Result<Vec<Document>> {
    let mut pending = vec![];
    for doc in self.documents().await?.into_iter() {
      if let Ok(accepted) = doc.in_accepted() {
        if accepted.bulletin().await?.is_published() {
          continue;
        }
      }

      pending.push(doc);
    }
    Ok(pending)
  }
  
  pub async fn has_accepted_docs(&self) -> Result<bool> {
    for doc in self.pending_docs().await?.into_iter() {
      if doc.is_accepted() {
        return Ok(true);
      }
    }

    Ok(false)
  }

  pub async fn save_evidence_and_model_changes<T: serde::Serialize>(
    &self,
    maybe_evidence: Vec<Vec<u8>>,
    maybe_model: Option<T>
  ) -> Result<()>  {
    let admin = self.org().await?.admin().await?;
  
    for evidence in &maybe_evidence {
      let doc = self.state.document().create_and_index(
        &self, evidence.as_ref(), None, admin.attrs.id, None, DocumentSource::Internal, true
      ).await;
      self.handle_uniqueness_error(doc)?;
    };

    if let Some(model) = maybe_model {
      let serialized = serde_json::to_string(&model)?.into_bytes();
      let doc = self.state.document().create_and_index(
        &self, &serialized, None, admin.attrs.id, None, DocumentSource::Internal, true
      ).await;
      self.handle_uniqueness_error(doc)?;
    };

    Ok(())
  }

  pub fn handle_uniqueness_error(&self, document: Result<Document>) -> Result<()>  {
    match document {
      Ok(_) => Ok(()),
      Err(Error::Validation { field, message }) => {
        match field.as_str() {
          "uniqueness" => Ok(()),
          _ => Err(Error::validation(&field, &message)),
        }
      },
      Err(x) => Err(x),
    }
  }
}

describe! {
  regtest!{ creates_story_for_document_and_can_append_new_documents (_site, c, mut chain)
    let alice = c.alice().await;
    let story = alice.story_with_signed_doc(b"alice", None, "").await;

    let first_snapshots = story.story_snapshots().await?;
    assert_that!( &story.hash().await?, rematch("[a-f0-9]{64}"));
    assert_eq!(first_snapshots[0].attrs.hash, story.hash().await?);
    
    let initial_docs = story.documents().await?;
    assert_eq!(initial_docs[0].attrs.org_id, *alice.org().await.id());
    assert_eq!(initial_docs[0].attrs.person_id, alice.person_id());

    assert_eq!(initial_docs.len(), 1);
    assert_eq!(story.published_documents().await?.len(), 0);

    chain.fund_signer_wallet();
    chain.simulate_stamping().await;

    assert_eq!(story.published_documents().await?.len(), 1);

    let bot = c.bot().await;
    bot.witnessed_email(&story, samples::multipart_email().as_bytes(), None).await;

    let last_docs = story.documents().await?;
    assert_eq!(last_docs[1].attrs.org_id, *alice.org().await.id());
    assert_eq!(last_docs[1].attrs.person_id, bot.person_id);
    assert_eq!(last_docs.len(), 2);
    assert_eq!(story.published_documents().await?.len(), 1);

    let last_snapshots = story.story_snapshots().await?;
    assert_eq!(last_snapshots[1].attrs.hash, story.hash().await?)
  }
  
  regtest!{ creates_multiple_stories (_site, c, mut chain)
    let alice = c.alice().await;
    let mut stories = vec![];
    for _ in 0..5 {
      stories.append(&mut alice.stories_with_signed_docs(b"alice").await);
    }

    for story in stories.iter() {
      alice.make_signed_document(story, b"message", None).await;
      alice.make_signed_document(story, b"message_2", None).await;
    }

    chain.fund_signer_wallet();
    chain.simulate_stamping().await;
    
    for story in stories {
      alice.make_signed_document(&story, b"message_3", None).await;
      alice.make_signed_document(&story, b"message_4", None).await;
    }
  }
}
