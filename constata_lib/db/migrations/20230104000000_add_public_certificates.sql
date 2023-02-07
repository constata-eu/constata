ALTER TABLE download_proof_links
  ADD COLUMN public_token VARCHAR,
  ADD COLUMN published_at TIMESTAMPTZ DEFAULT NULL;

ALTER TABLE access_tokens ALTER COLUMN auto_expires_on DROP NOT NULL;
ALTER TABLE certos_templates ADD COLUMN og_title_override VARCHAR;


CREATE FUNCTION "update_public_token_for_download_proof_links"() RETURNS boolean
    LANGUAGE plpgsql
AS
$$
DECLARE
    download_proof_link RECORD;
BEGIN

    FOR download_proof_link IN SELECT id, document_id FROM download_proof_links LOOP
      UPDATE download_proof_links SET public_token=CONCAT(download_proof_link.id, '-', download_proof_link.document_id) WHERE id=download_proof_link.id;
    END LOOP;

    RETURN TRUE;
END
$$;

select update_public_token_for_download_proof_links();


ALTER TABLE download_proof_links ALTER COLUMN public_token SET NOT NULL;