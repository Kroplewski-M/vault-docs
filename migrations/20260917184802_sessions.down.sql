-- Add down migration script here
ALTER TABLE users
    ADD COLUMN first_name VARVAR(100),
    ADD COLUMN last_name VARCHAR(100);

DROP TABLE IF EXISTS sessions;
