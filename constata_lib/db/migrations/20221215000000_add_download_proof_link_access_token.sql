ALTER TYPE access_token_kind ADD VALUE 'download_proof_link';
ALTER TABLE download_proof_links ADD COLUMN access_token_id INTEGER;

ALTER TABLE download_proof_links
  ADD FOREIGN KEY(access_token_id)
    REFERENCES access_tokens(id)
      ON DELETE CASCADE;

COMMIT;


CREATE FUNCTION "update_access_tokens_with_download_proof_links"() RETURNS boolean
    LANGUAGE plpgsql
AS
$$
DECLARE
    download_proof_link RECORD;
    organization_id INTEGER;
    admin_person RECORD;
    access_token_saved RECORD;
BEGIN

    FOR download_proof_link IN SELECT id, token, document_id FROM download_proof_links LOOP
      SELECT org_id INTO organization_id FROM documents WHERE id = download_proof_link.document_id;
      SELECT id INTO admin_person FROM persons WHERE org_id = organization_id AND admin LIMIT 1;
      INSERT INTO access_tokens (
        person_id,
        org_id,
        token,
        kind
      ) VALUES (
        admin_person.id,
        organization_id,
        download_proof_link.token,
        'download_proof_link'
      );
      UPDATE download_proof_links SET access_token_id=currval('access_tokens_id_seq'::regclass) WHERE id=download_proof_link.id;
    END LOOP;

    RETURN TRUE;
END
$$;

select update_access_tokens_with_download_proof_links();

ALTER TABLE download_proof_links
  ALTER COLUMN token DROP NOT NULL,
  ALTER COLUMN token SET DEFAULT NULL,
  ALTER COLUMN access_token_id SET NOT NULL,
  ALTER COLUMN valid_until DROP NOT NULL,
  ALTER COLUMN valid_until SET DEFAULT NULL;
