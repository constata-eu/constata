ALTER TABLE telegram_bot_outgoing_messages ADD COLUMN failed BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE telegram_bot_outgoing_messages ADD COLUMN error_log text;
