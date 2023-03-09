ALTER TABLE certos_entries RENAME COLUMN created_at TO received_at;
ALTER TABLE certos_entries ALTER COLUMN size_in_bytes DROP NOT NULL;
