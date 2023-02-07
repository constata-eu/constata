CREATE TYPE admin_role AS ENUM (
  'superadmin',
  'admin'
);

CREATE TABLE admin_users (
  id SERIAL PRIMARY KEY NOT NULL,
  username VARCHAR NOT NULL,
  hashed_password VARCHAR NOT NULL,
  otp_seed VARCHAR NOT NULL,
  role admin_role NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX admin_users_credentials ON admin_users (username, hashed_password);

CREATE TABLE admin_user_sessions (
  id SERIAL PRIMARY KEY NOT NULL,
  admin_user_id INTEGER NOT NULL,
  token VARCHAR NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  expired boolean DEFAULT FALSE NOT NULL
);

CREATE INDEX admin_user_sessions_token ON admin_user_sessions (token);
CREATE INDEX admin_user_sessions_retrieval ON admin_user_sessions (token, expired);
