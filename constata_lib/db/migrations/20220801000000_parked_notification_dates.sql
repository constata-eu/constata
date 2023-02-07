CREATE TABLE parked_reminders (
  id SERIAL PRIMARY KEY NOT NULL,
  person_id INTEGER NOT NULL,
  address VARCHAR NOT NULL,
  created_at TIMESTAMPTZ NOT NULL default now(),
  sent_at TIMESTAMPTZ
);
