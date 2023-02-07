ALTER TABLE email_bot_chats
    ADD FOREIGN KEY(first_document_id)
    REFERENCES documents(id)
    ON DELETE CASCADE;

ALTER TABLE email_bot_chats DROP CONSTRAINT fk_document;
ALTER TABLE email_callbacks DROP CONSTRAINT fk_document;
