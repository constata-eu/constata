ALTER TABLE email_addresses ADD COLUMN created_at TIMESTAMPTZ DEFAULT NULL;
UPDATE email_addresses e SET
  created_at = (select s.created_at FROM subscriptions s WHERE e.org_id = s.org_id);
ALTER TABLE email_addresses
  ALTER COLUMN created_at SET NOT NULL,
  ALTER COLUMN created_at SET DEFAULT now(),
  DROP CONSTRAINT email_addresses_address_key;

ALTER TYPE access_token_kind ADD VALUE 'invoice_link';
ALTER TABLE invoice_links ADD COLUMN access_token_id INTEGER;

ALTER TABLE invoice_links
  ADD FOREIGN KEY(access_token_id)
    REFERENCES access_tokens(id)
      ON DELETE CASCADE;

COMMIT;


CREATE FUNCTION "update_access_tokens_with_invoice_links"() RETURNS boolean
    LANGUAGE plpgsql
AS
$$
DECLARE
    invoice_link RECORD;
    admin_person RECORD;
    access_token_saved RECORD;
BEGIN

    FOR invoice_link IN SELECT id, token, org_id FROM invoice_links LOOP
      SELECT id INTO admin_person FROM persons WHERE org_id = invoice_link.org_id AND admin LIMIT 1;
      INSERT INTO access_tokens (
        person_id,
        org_id,
        token,
        kind
      ) VALUES (
        admin_person.id,
        invoice_link.org_id,
        invoice_link.token,
        'invoice_link'
      );
      UPDATE invoice_links SET access_token_id=currval('access_tokens_id_seq'::regclass) WHERE id=invoice_link.id;
    END LOOP;

    RETURN TRUE;
END
$$;

select update_access_tokens_with_invoice_links();

ALTER TABLE invoice_links ALTER COLUMN token DROP NOT NULL,
  ALTER COLUMN token SET DEFAULT NULL,
  ALTER COLUMN access_token_id SET NOT NULL;
