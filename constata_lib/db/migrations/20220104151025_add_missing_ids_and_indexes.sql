-- Add migration script here
ALTER TABLE email_callbacks ADD COLUMN id SERIAL PRIMARY KEY;
