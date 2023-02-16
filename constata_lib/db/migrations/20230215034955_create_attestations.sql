CREATE TABLE attestations (
  id SERIAL PRIMARY KEY NOT NULL,
  person_id INTEGER NOT NULL references persons(id),
  org_id INTEGER NOT NULL references orgs(id),
  story_id INTEGER NOT NULL references stories(id),
  markers TEXT NOT NULL DEFAULT '',
  created_at TIMESTAMPTZ NOT NULL default now(),
  deletion_id INTEGER references org_deletions(id)
);

CREATE INDEX attestations_person_id ON attestations (person_id);
CREATE INDEX attestations_story_id ON attestations (story_id);
CREATE INDEX attestations_org_id ON attestations (org_id);
