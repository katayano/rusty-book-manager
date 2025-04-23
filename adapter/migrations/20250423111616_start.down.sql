-- Add down migration script here

DROP TRIGGER IF EXISTS books_update_at_trigger ON books;
DROP TABLE IF EXISTS books;
DROP FUNCTION set_update_at();