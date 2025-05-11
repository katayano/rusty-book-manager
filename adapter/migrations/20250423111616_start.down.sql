-- Add down migration script here

DROP TRIGGER IF EXISTS books_update_at_trigger ON books;
DROP TABLE IF EXISTS books;
DROP TRIGGER IF EXISTS users_update_at_trigger ON users;
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS roles;
DROP FUNCTION set_update_at();