-- rolesテーブルに初期データを挿入
INSERT INTO roles (name) VALUES
    ('Admin'),
    ('User')
ON CONFLICT DO NOTHING;

-- usersテーブルに初期ユーザーを挿入
INSERT INTO users (name, email, password_hash, role_id) 
SELECT 
    'Admin User',
    'admin@example.com',
    '$2b$12$3HEDBSoxrKt/f6UXRiP0oOtfdWTwl6UF5mpHnSTi6xiH6hio9YnUG',
    role_id
FROM roles
WHERE name = 'Admin';
