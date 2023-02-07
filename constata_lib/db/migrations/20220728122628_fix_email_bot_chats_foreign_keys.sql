ALTER TABLE email_bot_chat_participants DROP CONSTRAINT fk_email_bot_chat;

ALTER TABLE email_bot_chat_participants
    ADD FOREIGN KEY(email_bot_chat_id)
    REFERENCES email_bot_chats(id)
    ON DELETE CASCADE;
