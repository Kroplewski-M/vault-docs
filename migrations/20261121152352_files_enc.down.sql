-- Add down migration script here
ALTER TABLE files
DROP COLUMN encrypted_key BYTEA,
DROP COLUMN nonce BYTEA,
DROP COLUMN enc_version;
