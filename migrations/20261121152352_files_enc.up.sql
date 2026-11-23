-- Add up migration script here
ALTER TABLE files
ADD encrypted_key BYTEA NOT NULL,
ADD nonce BYTEA NOT NULL,
ADD enc_version SMALLINT NOT NULL DEFAULT 1;
