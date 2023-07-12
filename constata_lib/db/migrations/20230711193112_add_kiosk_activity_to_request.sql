ALTER TABLE vc_requests ADD COLUMN last_active_at TIMESTAMPTZ;
ALTER TABLE vc_requests DROP COLUMN vc_presentation;
ALTER TABLE vc_requests DROP COLUMN vidchain_code;
ALTER TABLE vc_requests DROP COLUMN access_token_id;
DELETE FROM access_tokens WHERE kind = 'vc_request';
