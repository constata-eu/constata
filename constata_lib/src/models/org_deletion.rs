use super::*;

model!{
  state: Site,
  table: org_deletions,
  struct OrgDeletion {
    #[sqlx_model_hints(int4, default)]
    id: i32,
    #[sqlx_model_hints(int4)]
    org_id: i32,
    #[sqlx_model_hints(int4)]
    story_id: PersonId,
    #[sqlx_model_hints(timestamptz)]
    started_at: UtcDateTime,
    #[sqlx_model_hints(deletion_reason)]
    reason: DeletionReason,
    #[sqlx_model_hints(varchar)]
    description: String,
    #[sqlx_model_hints(boolean, default)]
    completed: bool,
    #[sqlx_model_hints(int4)]
    approving_admin_user: i32,
  },
  belongs_to {
    Org(org_id),
    Story(story_id),
  },
  has_many{
    Document(deletion_id),
    Person(deletion_id),
    Story(deletion_id),
    Pubkey(deletion_id),
    KycEndorsement(deletion_id),
    PubkeyDomainEndorsement(deletion_id),
    EmailAddress(deletion_id),
    Issuance(deletion_id),
    Template(deletion_id),
    Entry(deletion_id),
    DownloadProofLink(deletion_id),
  }
}

impl OrgDeletionHub {
  pub async fn delete_org(self,
    org_id: i32,
    approving_admin_user: i32,
    reason: DeletionReason,
    description: String,
    evidence: Vec<&[u8]>,
  ) -> ConstataResult<OrgDeletion> {
    let org = self.state.org().find(&org_id).await?;
    let evidence_vec = evidence.iter().map(|x| x.to_vec()).collect();

    if let Some(org_deletion) = org.org_deletion().await? {
      let story = org_deletion.story().await?;
      story.save_evidence_and_model_changes(evidence_vec, None::<()>).await?;
      org_deletion.set_relationships().await?;
      Ok(org_deletion.reloaded().await?)
    } else {
      let story = self.state.story().create(org_id, None, "delete org".to_string(), i18n::Lang::En).await?;
      let org_deletion = self.state.org_deletion().insert(InsertOrgDeletion {
        org_id,
        story_id: story.attrs.id,
        approving_admin_user,
        reason,
        description,
        started_at: Utc::now(),
      }).save().await?;
      story.save_evidence_and_model_changes(evidence_vec, Some(&org_deletion)).await?;
      org_deletion.set_relationships().await?;

      Ok(org_deletion.reloaded().await?)
    }
  }
}


impl OrgDeletion {
  pub async fn set_relationships(&self) -> ConstataResult<OrgDeletion> {
    macro_rules! set_deletion_id (
      ($vec_of_model:expr) => (
        for model in $vec_of_model {
          model.update().deletion_id(Some(self.attrs.id)).save().await?;
        }
    ));

    let org = self.org().await?;
    set_deletion_id![vec![org.clone()]];
    set_deletion_id![org.person_vec().await?];
    set_deletion_id![org.issuance_vec().await?];
    set_deletion_id![org.template_vec().await?];
    set_deletion_id![org.entry_vec().await?];
    set_deletion_id![org.pubkey_vec().await?];
    set_deletion_id![org.kyc_endorsement_vec().await?];
    set_deletion_id![org.pubkey_domain_endorsement_vec().await?];
    set_deletion_id![org.document_vec().await?];
    set_deletion_id![org.story_vec().await?];
    set_deletion_id![org.email_address_vec().await?];
    for m in org.document_vec().await? {
      set_deletion_id![m.download_proof_link_vec().await?];
    };

    Ok(self.clone())
  }

  pub async fn physical_delete(&self) -> ConstataResult<OrgDeletion> {
    let counter = self.state.org_deletion().select().completed_eq(true).count().await?;

    for r in self.issuance_vec().await? { r.storage_put(b"").await?  }

    for t in self.template_vec().await? { t.storage_put(b"").await?  }

    for e in self.entry_vec().await? { e.storage_put(b"").await?  }

    self.org().await?.update().public_name(None).save().await?; 

    for m in self.kyc_endorsement_vec().await? {
      m.update()
        .name(None).last_name(None).id_number(None).id_type(None).birthdate(None).country(None)
        .nationality(None).job_title(None).legal_entity_name(None).legal_entity_country(None)
        .legal_entity_registration(None).legal_entity_tax_id(None)
        .save().await?;
    };
    for m in self.pubkey_domain_endorsement_vec().await? {
      m.update().domain(format!("Deleted_{}", counter)).evidence(None).save().await?;
    };
    for m in self.email_address_vec().await? {
      m.clone().update().address(format!("Deleted_{}", counter)).save().await?;
    };

    Ok(self.clone().update().completed(true).save().await?)
  }

}

describe!{
  use crate::models::{
    email_callback::InsertEmailCallback,
  };

  dbtest!{ make_a_person_full_deletion(site, c)
    let bob = c.bob().await;
    let alice = c.alice().await;
    let robert = c.robert().await;
    let alice_id = alice.org().await.attrs.id;
    let email_address = alice.make_email("alice@gmail.com").await;
    alice.make_kyc_endorsement().await;
    alice.make_pubkey_domain_endorsement().await;
    let document = site.document().select().org_id_eq(alice_id).one().await?;
    document.get_or_create_download_proof_link(30).await?;
    site.email_callback().insert(InsertEmailCallback{
      document_id: document.attrs.id.clone(),
      address: email_address.attrs.address.clone(),
      custom_message: None,
      sent_at: None,
    }).save().await?;
    let template_file = read("template.zip");
    let template = alice.make_template(template_file).await;
    let issuance_file = read("issuance_one_entry.csv");
    alice.make_issuance(*template.id() ,issuance_file).await?;
    site.issuance().create_all_received().await?;

    assert_that!(site.org().find(&alice_id).await?.org_deletion().await?.is_none());
    let bob_kyc_endorsement = bob.make_kyc_endorsement().await;
    let robert_kyc_endorsement = robert.make_kyc_endorsement().await;

    let org_deletion = alice.make_org_deletion(b"testing org deletion").await;
    
    assert_that!(bob_kyc_endorsement.reloaded().await?.attrs.deletion_id.is_none());
    assert_that!(robert_kyc_endorsement.reloaded().await?.attrs.deletion_id.is_none());
    assert_that!(site.org().find(&alice_id).await?.org_deletion().await?.is_some());

    assert_eq!(org_deletion, site.org().find(&alice_id).await?.org_deletion().await?.unwrap());
    let story = org_deletion.story().await?;

    assert_eq!(2, story.document_vec().await?.len());

    alice.make_org_deletion(b"testing to make another").await;
    assert_eq!(3, story.document_vec().await?.len());
    
    alice.make_org_deletion(b"").await;
    assert_eq!(4, story.document_vec().await?.len());
    for doc in story.document_vec().await? {
      assert_eq!(doc.attrs.deletion_id, Some(org_deletion.attrs.id));
    };

    macro_rules! assert_count (($count:expr, $scope:expr) => ( assert_eq!($count, $scope.count().await?);));
    assert_count![1, org_deletion.issuance_scope()];
    assert_count![1, org_deletion.template_scope()];
    assert_count![1, org_deletion.entry_scope()];
    assert_count![1, org_deletion.pubkey_scope()];
    assert_count![1, org_deletion.kyc_endorsement_scope()];
    assert_count![1, org_deletion.pubkey_domain_endorsement_scope()];
    assert_count![1, org_deletion.email_address_scope()];
    assert_count![2, org_deletion.story_scope()];
    assert_count![5, org_deletion.document_scope()];
    assert_count![1, org_deletion.download_proof_link_scope()];

    let mut org = org_deletion.org().await?;
    for m in org.issuance_vec().await? {
      assert_that!(!m.storage_fetch().await?.is_empty());
    };
    for m in org.template_vec().await? {
      assert_that!(!m.storage_fetch().await?.is_empty());
    };
    for m in org.entry_vec().await? {
      assert_that!(!m.storage_fetch().await?.is_empty());
    };

    let physical_deletion = org_deletion.physical_delete().await?;
    
    org = physical_deletion.org().await?;
    assert_that!(physical_deletion.attrs.completed);
    for m in org.issuance_vec().await? {
      assert_that!(m.storage_fetch().await?.is_empty());
    };
    for m in org.template_vec().await? {
      assert_that!(m.storage_fetch().await?.is_empty());
    };
    for m in org.entry_vec().await? {
      assert_that!(m.storage_fetch().await?.is_empty());
    };
    for m in org.kyc_endorsement_vec().await? {
      assert_that!(m.attrs.name.is_none());
      assert_that!(m.attrs.last_name.is_none());
      assert_that!(m.attrs.id_number.is_none());
      assert_that!(m.attrs.id_type.is_none());
      assert_that!(m.attrs.birthdate.is_none());
      assert_that!(m.attrs.country.is_none());
      assert_that!(m.attrs.nationality.is_none());
      assert_that!(m.attrs.job_title.is_none());
      assert_that!(m.attrs.legal_entity_name.is_none());
      assert_that!(m.attrs.legal_entity_country.is_none());
      assert_that!(m.attrs.legal_entity_registration.is_none());
      assert_that!(m.attrs.legal_entity_tax_id.is_none());
    };
    for m in org.pubkey_domain_endorsement_vec().await? {
      assert_eq!("Deleted_0".to_string(), m.attrs.domain);
      assert_that!(m.attrs.evidence.is_none());
    };
    for m in org.email_address_vec().await? {
      assert_eq!("Deleted_0".to_string(), m.attrs.address);
    };
  }
}
