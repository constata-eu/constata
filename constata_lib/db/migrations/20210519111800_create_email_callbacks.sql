CREATE TABLE email_callbacks (
  document_id VARCHAR UNIQUE NOT NULL,
  address VARCHAR NOT NULL,
  sent_at TIMESTAMPTZ,
  CONSTRAINT fk_document
    FOREIGN KEY(document_id) 
      REFERENCES documents(id)
);

CREATE INDEX email_callbacks_document_id ON email_callbacks (document_id);
CREATE INDEX email_callbacks_sent_at ON email_callbacks (sent_at);
