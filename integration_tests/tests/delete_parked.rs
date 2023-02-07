constata_lib::describe_one! {
  use integration_tests::*;

  integration_test!{ can_delete_parked_document (c, d)
    let bot = c.bot().await;
    let mut document_email = bot.witnessed_email_with_story().await;

    d.goto(&document_email.get_or_create_delete_parked_url().await?).await;
    d.wait_for_text(".mb-3", r"Dismiss parked document.*").await;
    d.click(".btn-accept").await;
    d.click(".btn-confirm").await;
    d.wait_for_text(".modal-body p", r"You have desisted from certifying.*").await;
    assert_that!(document_email.reload().await.is_err());
  }

  integration_test!{ cannot_delete_accepted_document (c, d)
    let alice = c.alice().await;
    let funded_doc = alice.signed_document(b"hello world").await;

    d.goto(&funded_doc.get_or_create_delete_parked_url().await?).await;
    d.wait_for_text(".mb-3", r"This document cannot be dismissed.*").await;
  }

  integration_test!{ cannot_delete_document_with_bad_token (c, d)
    let enterprise = c.enterprise().await;
    let mut unfunded_document = enterprise.signed_document(b"hello world").await;

    d.goto(&unfunded_document.get_or_create_delete_parked_url().await?).await;
    d.wait_for_text(".mb-3", r"Dismiss parked document*").await;
    unfunded_document.clone().update().delete_parked_token(Some("other+token".to_string())).save().await?;
    d.click(".btn-accept").await;
    d.click(".btn-confirm").await;
    d.wait_for_text(".modal-body p", r"An unexpected error ocurred*").await;
    d.click(".modal.error .btn-back").await;
    assert_that!(unfunded_document.reload().await.ok().is_some());
  }
}
