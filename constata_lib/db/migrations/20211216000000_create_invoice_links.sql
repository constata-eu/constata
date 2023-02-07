CREATE TABLE invoice_links (
  id SERIAL PRIMARY KEY NOT NULL,
  token VARCHAR NOT NULL,
  person_id INTEGER NOT NULL,
  invoice_id INTEGER
);

CREATE INDEX invoice_links_token ON invoice_links (token);
