ALTER TABLE document_parts ALTER COLUMN payload DROP NOT NULL;
ALTER TABLE certos_entries ALTER COLUMN payload DROP NOT NULL;
ALTER TABLE certos_requests ALTER COLUMN payload DROP NOT NULL;
ALTER TABLE certos_templates ALTER COLUMN payload DROP NOT NULL;
ALTER TABLE telegram_bot_updates ALTER COLUMN payload DROP NOT NULL;
