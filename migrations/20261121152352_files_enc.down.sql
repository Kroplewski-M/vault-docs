-- Add down migration script here
ALTER TABLE files
DROP COLUMN encrypted_key,
DROP COLUMN nonce,
DROP COLUMN enc_version;
