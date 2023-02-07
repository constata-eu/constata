ALTER TABLE email_callbacks DROP CONSTRAINT email_callbacks_document_id_key;
ALTER TABLE email_callbacks ADD CONSTRAINT email_callbacks_document_id_key UNIQUE (document_id, address, cc);


