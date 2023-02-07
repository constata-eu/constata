CREATE TABLE email_bot_chats (
  id SERIAL PRIMARY KEY NOT NULL,
  story_id INTEGER NOT NULL,
  first_document_id VARCHAR NOT NULL,
  person_id INTEGER NOT NULL,
  thread_id VARCHAR UNIQUE NOT NULL,
  message_id VARCHAR UNIQUE NOT NULL,
  subject VARCHAR NOT NULL,
  greeting_sent_at TIMESTAMPTZ,
  CONSTRAINT fk_document
    FOREIGN KEY(first_document_id) 
      REFERENCES documents(id),
  CONSTRAINT fk_story
    FOREIGN KEY(story_id) 
      REFERENCES stories(id),
  CONSTRAINT fk_person
    FOREIGN KEY(person_id) 
      REFERENCES persons(id),
  UNIQUE(person_id,thread_id)
);

CREATE TABLE email_bot_chat_participants (
  id SERIAL PRIMARY KEY NOT NULL,
  email_bot_chat_id INTEGER NOT NULL,
  address TEXT NOT NULL,
  CONSTRAINT fk_email_bot_chat
    FOREIGN KEY(email_bot_chat_id) 
      REFERENCES email_bot_chats(id)
);

CREATE INDEX email_bot_chats_story_id ON email_bot_chats (story_id);
CREATE INDEX email_bot_chats_person_id ON email_bot_chats (person_id);
CREATE INDEX email_bot_chats_first_document_id ON email_bot_chats (first_document_id);
CREATE INDEX email_bot_chats_greeting_sent_at ON email_bot_chats (greeting_sent_at);
