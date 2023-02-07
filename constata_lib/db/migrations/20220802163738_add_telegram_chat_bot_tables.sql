CREATE TABLE telegram_bot_updates (
  id SERIAL PRIMARY KEY NOT NULL,
  update_id INTEGER NOT NULL,
  synced_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  processed_at TIMESTAMPTZ,
  payload TEXT NOT NULL
);

CREATE TABLE telegram_bot_outgoing_messages (
  id SERIAL PRIMARY KEY NOT NULL,
  chat_id BIGINT NOT NULL,
  message TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  sent_at TIMESTAMPTZ,
  telegram_id INTEGER
);

CREATE TABLE telegram_users (
  id VARCHAR NOT NULL,
  person_id INTEGER NOT NULL,
  first_name VARCHAR NOT NULL,
  username VARCHAR,
  last_name VARCHAR,
  CONSTRAINT fk_person
    FOREIGN KEY(person_id) 
      REFERENCES persons(id)
);

CREATE TABLE telegram_bot_group_chats (
  id SERIAL PRIMARY KEY NOT NULL,
  story_id INTEGER NOT NULL,
  person_id INTEGER NOT NULL,
  chat_id BIGINT NOT NULL,
  first_document_id VARCHAR,
  greeting_message_id INTEGER,
  last_greeting_reminder TIMESTAMPTZ,

  CONSTRAINT fk_document
    FOREIGN KEY(first_document_id) 
      REFERENCES documents(id) ON DELETE CASCADE,
  CONSTRAINT fk_story
    FOREIGN KEY(story_id) 
      REFERENCES stories(id) ON DELETE CASCADE,
  CONSTRAINT fk_person
    FOREIGN KEY(person_id) 
      REFERENCES persons(id)
);

CREATE TABLE telegram_bot_private_chats (
  id SERIAL PRIMARY KEY NOT NULL,
  person_id INTEGER NOT NULL,
  chat_id BIGINT NOT NULL,
  greeting_message_id INTEGER NOT NULL,

  CONSTRAINT fk_person
    FOREIGN KEY(person_id) 
      REFERENCES persons(id)
);

CREATE TABLE telegram_bot_private_chat_documents (
  id SERIAL PRIMARY KEY NOT NULL,
  document_id VARCHAR NOT NULL,
  person_id INTEGER NOT NULL,
  notification_message_id INTEGER,
  CONSTRAINT fk_document
    FOREIGN KEY(document_id) 
      REFERENCES documents(id) ON DELETE CASCADE,
  CONSTRAINT fk_person
    FOREIGN KEY(person_id) 
      REFERENCES persons(id)
);

CREATE TYPE document_source AS ENUM (
  'email',
  'api',
  'telegram',
  'internal'
);

ALTER TABLE documents ADD COLUMN sourced_from document_source;

UPDATE documents d
SET sourced_from = 'email'
WHERE EXISTS (SELECT * FROM witnessed_documents wd WHERE wd.document_id = d.id);

UPDATE documents SET sourced_from = 'api' WHERE sourced_from IS NULL;

ALTER TABLE documents ALTER COLUMN sourced_from SET NOT NULL;
