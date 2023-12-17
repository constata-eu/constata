-- These tables where used by legacy functionality.
DROP TABLE approval_acks;
DROP TABLE approval_requests;
DROP TABLE approvals;
DROP TABLE biometric_proofs;
DROP TABLE create_email_credentials_tokens;
DROP TABLE email_bot_chat_participants;
DROP TABLE email_bot_chats;
DROP TABLE expenses;
DROP TABLE fundings;
DROP TABLE signature_acks;
DROP TABLE signature_requests;
DROP TABLE signatures;
DROP TABLE signed_documents;
DROP TABLE subscription_invoice;
DROP TABLE telegram_bot_group_chats;
DROP TABLE telegram_bot_outgoing_messages;
DROP TABLE telegram_bot_private_chat_documents;
DROP TABLE telegram_bot_private_chats;
DROP TABLE telegram_bot_updates;
DROP TABLE telegram_users;
DROP TABLE witnessed_documents;

-- Document part payloads are not stored in the database.
ALTER TABLE document_parts DROP COLUMN payload;

-- Certos entries are now just entries.
ALTER INDEX certos_entries_pkey  RENAME TO entries_pkey;
ALTER INDEX certos_entries_deletion_id RENAME TO entries_deletion_id;
ALTER INDEX certos_entries_org_id RENAME TO entries_org_id;
ALTER INDEX certos_entries_person_id RENAME TO entries_person_id;
ALTER INDEX certos_entries_request_id RENAME TO entries_issuance_id;
ALTER INDEX certos_entries_state RENAME TO entries_state;

ALTER SEQUENCE certos_entries_id_seq RENAME TO entries_id_seq;

ALTER TABLE certos_entries DROP COLUMN payload;
DROP INDEX certos_entries_app_id;
ALTER TABLE certos_entries DROP COLUMN app_id;
ALTER TABLE certos_entries RENAME COLUMN request_id TO issuance_id;

ALTER TABLE certos_entries RENAME CONSTRAINT "certos_entries_deletion_id_fkey" TO "entries_deletion_id_fkey";
ALTER TABLE certos_entries RENAME CONSTRAINT "certos_entries_document_id_fkey" TO "entries_document_id_fkey";
ALTER TABLE certos_entries RENAME TO entries;

-- Certos templates are now just templates.
ALTER INDEX certos_templates_pkey  RENAME TO templates_pkey;
ALTER INDEX certos_templates_org_id RENAME TO templates_org_id;
ALTER INDEX certos_templates_person_id RENAME TO templates_person_id;

ALTER SEQUENCE certos_templates_id_seq RENAME TO templates_id_seq;

ALTER TABLE certos_templates DROP COLUMN payload;
ALTER TABLE certos_templates DROP COLUMN size_in_bytes;
DROP INDEX certos_templates_app_id;
ALTER TABLE certos_templates DROP COLUMN app_id;

ALTER TABLE certos_templates RENAME TO templates;

-- Certos requests are now Issuances.
ALTER INDEX certos_requests_pkey  RENAME TO issuances_pkey;
ALTER INDEX certos_requests_deletion_id RENAME TO issuances_deletion_id;
ALTER INDEX certos_requests_org_id RENAME TO issuances_org_id;
ALTER INDEX certos_requests_person_id RENAME TO issuances_person_id;
ALTER INDEX certos_requests_state RENAME TO issuances_state;

ALTER SEQUENCE certos_requests_id_seq RENAME TO issuances_id_seq;

ALTER TABLE certos_requests DROP COLUMN payload;
ALTER TABLE certos_requests DROP COLUMN size_in_bytes;
DROP INDEX certos_requests_app_id;
ALTER TABLE certos_requests DROP COLUMN app_id;

ALTER TABLE certos_requests RENAME CONSTRAINT "certos_requests_deletion_id_fkey" TO "issuances_deletion_id_fkey";
ALTER TABLE certos_requests RENAME TO issuances;

-- Certos apps should be merged with organizations.
-- Certos apps just bind an organization to an app.
-- If anything that points to the app is already pointing to the organization they can be safely removed.
DROP TABLE certos_apps;
