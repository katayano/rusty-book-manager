-- Add up migration script here

-- update_atを自動更新するトリガーを作成
CREATE OR REPLACE FUNCTION set_update_at() RETURNS TRIGGER AS '
    BEGIN
        new.update_at := ''now()'';
        return new;
    END;
' LANGUAGE 'plpgsql';

-- booksテーブルの作成
CREATE TABLE IF NOT EXISTS books (
    book_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    author VARCHAR(255) NOT NULL,
    isbn VARCHAR(255) NOT NULL,
    description VARCHAR(1024) NOT NULL,
    created_at TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    update_at TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3)
);

-- booksテーブルへのトリガー追加
CREATE TRIGGER books_update_at_trigger
    BEFORE UPDATE ON books FOR EACH ROW
    EXECUTE PROCEDURE set_update_at();