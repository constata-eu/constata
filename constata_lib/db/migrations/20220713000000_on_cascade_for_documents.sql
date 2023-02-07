ALTER TABLE approval_requests
    ADD FOREIGN KEY(document_id)
    REFERENCES documents(id)
    ON DELETE CASCADE;

ALTER TABLE certos_entries
    ADD FOREIGN KEY(document_id)
    REFERENCES documents(id)
    ON DELETE CASCADE;

ALTER TABLE download_proof_links
    ADD FOREIGN KEY(document_id)
    REFERENCES documents(id)
    ON DELETE CASCADE;

ALTER TABLE email_bot_chats
    ADD FOREIGN KEY(first_document_id)
    REFERENCES documents(id)
    ON DELETE CASCADE;

ALTER TABLE email_callbacks
    ADD FOREIGN KEY(document_id)
    REFERENCES documents(id)
    ON DELETE CASCADE;

ALTER TABLE signature_requests
    ADD FOREIGN KEY(document_id)
    REFERENCES documents(id)
    ON DELETE CASCADE;

ALTER TABLE signed_documents
    ADD FOREIGN KEY(document_id)
    REFERENCES documents(id)
    ON DELETE CASCADE;