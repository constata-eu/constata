CREATE TYPE language AS ENUM (
  'en',
  'es'
);

ALTER TABLE persons ADD COLUMN lang language NOT NULL default 'es';
ALTER TABLE persons ADD COLUMN lang_set_from VARCHAR NOT NULL default 'database_default';
ALTER TABLE stories ADD COLUMN lang language NOT NULL default 'es';
