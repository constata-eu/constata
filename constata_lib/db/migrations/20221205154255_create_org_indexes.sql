ALTER TABLE persons ADD FOREIGN KEY(org_id) REFERENCES orgs(id);
CREATE INDEX persons_org_id ON persons (org_id);

ALTER TABLE persons ADD FOREIGN KEY(deletion_id) REFERENCES org_deletions(id);
CREATE INDEX persons_deletion_id ON persons (deletion_id);

CREATE INDEX persons_suspended ON persons (suspended);
CREATE INDEX persons_admin ON persons (admin);
CREATE INDEX persons_billing ON persons (billing);

ALTER TABLE certos_requests ADD FOREIGN KEY(deletion_id) REFERENCES org_deletions(id);
CREATE INDEX certos_requests_deletion_id ON certos_requests (deletion_id);

ALTER TABLE certos_entries ADD FOREIGN KEY(deletion_id) REFERENCES org_deletions(id);
CREATE INDEX certos_entries_deletion_id ON certos_entries (deletion_id);

ALTER TABLE documents ADD FOREIGN KEY(deletion_id) REFERENCES org_deletions(id);
CREATE INDEX documents_deletion_id ON documents (deletion_id);

ALTER TABLE documents ADD FOREIGN KEY(gift_id) REFERENCES gifts(id);
CREATE INDEX documents_gift_id ON documents (gift_id);

CREATE INDEX documents_delete_parked_token ON documents (delete_parked_token);
CREATE INDEX documents_sourced_from ON documents (sourced_from);
CREATE INDEX documents_funded ON documents (funded);
CREATE INDEX documents_funded_at ON documents (funded_at);

ALTER TABLE orgs ADD FOREIGN KEY(subscription_id) REFERENCES subscriptions(id);
CREATE INDEX orgs_subscription_id ON orgs (subscription_id);

ALTER TABLE orgs ADD FOREIGN KEY(deletion_id) REFERENCES org_deletions(id);
CREATE INDEX orgs_deletion_id ON orgs (deletion_id);

CREATE INDEX orgs_stripe_customer_id ON orgs (stripe_customer_id);
CREATE INDEX orgs_public_name ON orgs (public_name);
CREATE INDEX orgs_logo_url ON orgs (logo_url);
CREATE INDEX orgs_created_at ON orgs (created_at);

CREATE INDEX bulletins_started_at ON bulletins (started_at);
CREATE INDEX bulletins_block_hash ON bulletins (block_hash);
CREATE INDEX bulletins_submitted_at ON bulletins (submitted_at);
CREATE INDEX bulletins_block_time ON bulletins (block_time);

ALTER TABLE pubkeys ADD FOREIGN KEY(person_id) REFERENCES persons(id);
CREATE INDEX pubkeys_person_id ON pubkeys (person_id);

ALTER TABLE kyc_endorsements ADD FOREIGN KEY(person_id) REFERENCES persons(id);
CREATE INDEX kyc_endorsements_person_id ON kyc_endorsements (person_id);

ALTER TABLE kyc_endorsements ADD FOREIGN KEY(story_id) REFERENCES stories(id);
CREATE INDEX kyc_endorsements_story_id ON kyc_endorsements (story_id);

CREATE INDEX kyc_endorsements_created_at ON kyc_endorsements (created_at);

ALTER TABLE email_addresses ADD FOREIGN KEY(person_id) REFERENCES persons(id);
CREATE INDEX email_addresses_person_id ON email_addresses (person_id);

ALTER TABLE email_addresses ADD FOREIGN KEY(deletion_id) REFERENCES org_deletions(id);
CREATE INDEX email_addresses_deletion_id ON email_addresses (deletion_id);

CREATE INDEX email_addresses_access_token_id ON email_addresses (access_token_id);
CREATE INDEX email_addresses_verified_at ON email_addresses (verified_at);
CREATE INDEX email_addresses_keep_private ON email_addresses (keep_private);

ALTER TABLE stories ADD FOREIGN KEY(deletion_id) REFERENCES org_deletions(id);
CREATE INDEX stories_deletion_id ON stories (deletion_id);

ALTER TABLE invoice_links ADD FOREIGN KEY(org_id) REFERENCES orgs(id);
CREATE INDEX invoice_links_org_id ON invoice_links (org_id);

ALTER TABLE invoice_links ADD FOREIGN KEY(invoice_id) REFERENCES invoices(id);
CREATE INDEX invoice_links_invoice_id ON invoice_links (invoice_id);

ALTER TABLE parked_reminders ADD FOREIGN KEY(org_id) REFERENCES orgs(id);
CREATE INDEX parked_reminders_org_id ON parked_reminders (org_id);

ALTER TABLE org_deletions ADD FOREIGN KEY(story_id) REFERENCES stories(id);
CREATE INDEX org_deletions_story_id ON org_deletions (story_id);
