ALTER TABLE documents ADD COLUMN delete_parked_token VARCHAR DEFAULT NULL;

ALTER TABLE document_parts
    ADD FOREIGN KEY(document_id)
    REFERENCES documents(id)
    ON DELETE CASCADE;

ALTER TABLE document_part_signatures
    ADD FOREIGN KEY(document_part_id)
    REFERENCES document_parts(id)
    ON DELETE CASCADE;

ALTER TABLE witnessed_documents
    ADD FOREIGN KEY(document_id)
    REFERENCES documents(id)
    ON DELETE CASCADE;
