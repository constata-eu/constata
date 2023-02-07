-- Add migration script here
BEGIN;
LOCK TABLE orgs IN EXCLUSIVE MODE;
SELECT setval('orgs_id_seq'::regclass, COALESCE((SELECT MAX(id)+1 FROM orgs), 1), false);
COMMIT;
