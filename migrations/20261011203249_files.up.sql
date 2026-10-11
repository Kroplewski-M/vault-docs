-- Add up migration script here
CREATE TABLE files
(
  id UUID PRIMARY KEY, -- used as the name in AWS dont want to auto generate it in sql
  original_name TEXT NOT NULL,
  ext VARCHAR(30) NOT NULL,
  mime_type VARCHAR(100) NOT NULL,
  size BIGINT NOT NULL CHECK (size >= 0),
  user_id UUID NOT NULL REFERENCES users(id),
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_files_user_id ON files (user_id);
