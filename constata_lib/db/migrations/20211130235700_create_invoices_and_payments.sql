CREATE TYPE payment_source AS ENUM (
  'bankbbva',
  'stripe',
  'btcpay',
  'santaclaus'
);

ALTER TABLE persons ADD COLUMN subscription_id INTEGER;
ALTER TABLE persons ADD COLUMN stripe_customer_id VARCHAR;

CREATE TABLE subscriptions (
  id SERIAL PRIMARY KEY NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  invoicing_day INTEGER NOT NULL,
  person_id INTEGER NOT NULL,
  is_active BOOLEAN NOT NULL DEFAULT FALSE, 
  plan_name VARCHAR NOT NULL,
  max_monthly_gift DECIMAL NOT NULL,
  required_token_purchase DECIMAL NOT NULL,
  price_per_token DECIMAL NOT NULL,
  default_payment_source payment_source,
  stripe_subscription_id VARCHAR
);

CREATE INDEX subscription_person_id ON subscriptions (person_id);
CREATE INDEX subscription_is_active ON subscriptions (is_active);

CREATE TABLE subscription_invoice (
  id SERIAL PRIMARY KEY NOT NULL,
  billing_period TIMESTAMPTZ NOT NULL,
  subscription_id INTEGER NOT NULL,
  person_id INTEGER NOT NULL,
  invoice_id INTEGER NOT NULL,
  paid BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX subscription_invoice_paid ON subscription_invoice (paid);
CREATE INDEX subscription_invoice_subscription_id ON subscription_invoice (subscription_id);
CREATE INDEX subscription_invoice_person_id ON subscription_invoice (person_id);

CREATE TABLE gifts (
  id SERIAL PRIMARY KEY NOT NULL,
  person_id INTEGER NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  tokens DECIMAL NOT NULL,
  reason TEXT NOT NULL
);
CREATE INDEX gift_person_id ON gifts (person_id);

ALTER TABLE documents ADD COLUMN gift_id INTEGER;
ALTER TABLE documents ALTER COLUMN cost TYPE DECIMAL;

CREATE TABLE payments (
  id SERIAL PRIMARY KEY NOT NULL,
  person_id INTEGER NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  amount DECIMAL NOT NULL,
  tokens DECIMAL NOT NULL,
  fees DECIMAL NOT NULL,
  payment_source payment_source NOT NULL,
  clearing_data TEXT NOT NULL,
  invoice_id INTEGER
);
CREATE INDEX payments_student_id ON payments (person_id);
CREATE INDEX payments_invoice_id ON payments (invoice_id);
CREATE INDEX payments_payment_source ON payments (payment_source);

CREATE TABLE invoices (
  id SERIAL PRIMARY KEY NOT NULL,
  person_id INTEGER NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  amount DECIMAL NOT NULL,
  tokens DECIMAL NOT NULL,
  payment_source payment_source NOT NULL,
  description TEXT NOT NULL,
  external_id VARCHAR NOT NULL,
  url TEXT NOT NULL,
  notified_on TIMESTAMPTZ,
  paid BOOLEAN NOT NULL DEFAULT FALSE,
  payment_id INTEGER,
  expired BOOLEAN NOT NULL DEFAULT FALSE
);
CREATE INDEX invoices_person_id ON invoices (person_id);
CREATE INDEX invoices_payment_id ON invoices (payment_id);
CREATE INDEX invoices_payment_source ON invoices (payment_source);

INSERT INTO subscriptions (
  person_id,
  invoicing_day,
  plan_name,
  max_monthly_gift,
  required_token_purchase,
  price_per_token )
  SELECT id, 1, 'Early Bird', 10, 0, 0.5 FROM persons;

INSERT INTO gifts (
  person_id,
  created_at,
  tokens,
  reason
  )
  SELECT
  persons.id,
  now(),
  (SELECT COALESCE(SUM(cost),0)::decimal FROM documents WHERE documents.person_id = persons.id AND documents.funded),
  'Digital Inclusion' FROM persons;

UPDATE persons SET subscription_id = (SELECT id FROM subscriptions WHERE subscriptions.person_id = persons.id);
