use async_trait::async_trait;
use derive_new::new;

use kernel::model::{
    id::UserId,
    role::Role,
    user::{
        User,
        event::{CreateUser, DeleteUser, UpdateUserPassword, UpdateUserRole},
    },
};
use kernel::repository::user::UserRepository;
use shared::error::{AppError, AppResult};

use crate::database::{ConnectionPool, model::user::UserRow};

#[derive(new)]
pub struct UserRepositoryImpl {
    db: ConnectionPool,
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    /// ユーザーを登録する
    async fn create(&self, event: CreateUser) -> AppResult<User> {
        let user_id = UserId::new();
        let hashed_password = hash_password(&event.password)?;
        // ユーザー追加時の権限は一般のユーザー権限とする
        let role = Role::User;

        let res = sqlx::query!(
            r#"
            INSERT INTO users (user_id, name, email, password_hash, role_id)
            SELECT $1, $2, $3, $4, role_id FROM roles WHERE name = $5
            "#,
            user_id as _,
            event.name,
            event.email,
            hashed_password,
            role.as_ref(),
        )
        .execute(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() == 0 {
            return Err(AppError::NoRowsAffectedError(
                "No user has been created".to_string(),
            ));
        }

        // ユーザー登録に成功した場合、Userオブジェクトを返す
        Ok(User {
            id: user_id,
            name: event.name,
            email: event.email,
            role,
        })
    }
    /// ユーザーを全件取得する
    async fn find_all(&self) -> AppResult<Vec<User>> {
        let users = sqlx::query_as!(
            UserRow,
            r#"
            SELECT u.user_id, u.name, u.email, r.name as role_name, u.created_at, u.updated_at
            FROM users AS u
            INNER JOIN roles AS r USING(role_id)
            ORDER BY u.created_at DESC
            "#,
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?
        .into_iter()
        .filter_map(|row| User::try_from(row).ok())
        .collect();

        Ok(users)
    }
    /// ユーザーを取得する
    async fn find_current_user(&self, current_user_id: UserId) -> AppResult<Option<User>> {
        let row = sqlx::query_as!(
            UserRow,
            r#"
            SELECT u.user_id, u.name, u.email, r.name as role_name, u.created_at, u.updated_at
            FROM users AS u
            INNER JOIN roles AS r USING(role_id)
            WHERE u.user_id = $1
            "#,
            current_user_id as _,
        )
        .fetch_optional(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        match row {
            Some(user) => Ok(Some(User::try_from(user)?)),
            None => Ok(None),
        }
    }
    /// ユーザーを削除する
    async fn delete(&self, event: DeleteUser) -> AppResult<()> {
        let res = sqlx::query!(
            r#"
            DELETE FROM users WHERE user_id = $1
            "#,
            event.id as _,
        )
        .execute(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() == 0 {
            return Err(AppError::EntityNotFound(
                "Specified user not found".to_string(),
            ));
        }

        Ok(())
    }
    /// ユーザーのロールを更新する
    async fn update_role(&self, event: UpdateUserRole) -> AppResult<()> {
        let res = sqlx::query!(
            r#"
            UPDATE users SET role_id = (SELECT role_id FROM roles WHERE name = $1), updated_at = NOW()
            WHERE user_id = $2
            "#,
            event.role.as_ref(),
            event.id as _,
        )
        .execute(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() == 0 {
            return Err(AppError::EntityNotFound(
                "Specified user not found".to_string(),
            ));
        }

        Ok(())
    }
    /// ユーザーのパスワードを更新する
    async fn update_password(&self, event: UpdateUserPassword) -> AppResult<()> {
        let mut tx = self.db.begin().await?;

        // 現在のパスワードを取得する
        let original_password_hash = sqlx::query!(
            r#"
            SELECT password_hash FROM users WHERE user_id = $1
            "#,
            event.id as _,
        )
        // Transactionをデリファレンスし、可変参照を取得。それを使ってクエリを実行
        .fetch_one(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?
        .password_hash;

        // 現在のパスワードが正しいか確認
        verify_password(&event.current_password, &original_password_hash)?;

        // 新しいパスワードをハッシュ化し、更新する
        let new_password_hash = hash_password(&event.new_password)?;
        sqlx::query!(
            r#"
            UPDATE users SET password_hash = $1, updated_at = NOW() WHERE user_id = $2
            "#,
            new_password_hash,
            event.id as _,
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?;

        // トランザクションをコミットする
        tx.commit().await.map_err(AppError::TransactionError)?;
        Ok(())
    }
}

fn hash_password(password: &str) -> AppResult<String> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(AppError::from)
}

fn verify_password(password: &str, hashed: &str) -> AppResult<()> {
    let valid = bcrypt::verify(password, hashed)?;
    if !valid {
        return Err(AppError::UnauthenticatedError);
    }
    Ok(())
}
