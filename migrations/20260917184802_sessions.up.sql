-- Add up migration script here
 CREATE TABLE sessions (
   id UUID PRIMARY KEY,
   user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
   expires_at TIMESTAMPTZ NOT NULL,
   created_at TIMESTAMPTZ NOT NULL DEFAULT now()
 );
 CREATE INDEX idx_sessions_expires_at ON sessions (expires_at);

ALTER TABLE users
    DROP COLUMN first_name,
    DROP COLUMN last_name;; 
