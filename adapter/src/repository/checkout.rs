//! 蔵書の貸し出しのDB操作のための具象実装をするモジュール

use async_trait::async_trait;
use derive_new::new;

use kernel::model::{
    checkout::{
        Checkout,
        event::{CreateCheckout, UpdateReturned},
    },
    id::{BookId, CheckoutId, UserId},
};
use kernel::repository::checkout::CheckoutRepository;
use shared::error::{AppError, AppResult};

use crate::database::{
    ConnectionPool,
    model::checkout::{CheckoutRow, CheckoutStateRow, ReturnedCheckoutRow},
};

#[derive(new)]
pub struct CheckoutRepositoryImpl {
    db: ConnectionPool,
}

#[async_trait]
impl CheckoutRepository for CheckoutRepositoryImpl {
    /// 貸出操作を行う
    async fn create(&self, event: CreateCheckout) -> AppResult<()> {
        let mut tx = self.db.begin().await?;

        // トランザクション分離レベルをSERIALIZABLEに設定
        self.set_transaction_serializable(&mut tx).await?;

        // 事前に以下をチェック
        // - 指定の蔵書が存在するか
        // - 指定の蔵書が貸出中でないか
        {
            let res = sqlx::query_as!(
                CheckoutStateRow,
                r#"
                    SELECT 
                        b.book_id, 
                        c.checkout_id AS "checkout_id?: CheckoutId", 
                        NULL AS "user_id?: UserId"
                    FROM books AS b
                    LEFT OUTER JOIN checkouts AS c USING(book_id)
                    WHERE book_id = $1
                "#,
                event.book_id as _
            )
            .fetch_optional(&mut *tx)
            .await
            .map_err(AppError::SpecificOperationError)?;

            match res {
                // 蔵書が存在しない場合
                None => {
                    return Err(AppError::EntityNotFound(format!(
                        "書籍（{}）が見つかりませんでした",
                        event.book_id
                    )));
                }
                // 蔵書は存在するが貸出中の場合
                Some(CheckoutStateRow {
                    checkout_id: Some(_),
                    ..
                }) => {
                    return Err(AppError::UnprocessableEntity(format!(
                        "書籍（{}）に対する貸出が既に存在します",
                        event.book_id
                    )));
                }
                // それ以外は処理を続行
                _ => {}
            }
        }

        // 貸出情報を登録
        let checkout_id = CheckoutId::new();
        let res = sqlx::query!(
            r#"
            INSERT INTO checkouts (checkout_id, book_id, user_id, checked_out_at)
            VALUES ($1, $2, $3, $4)
            "#,
            checkout_id as _,
            event.book_id as _,
            event.checked_out_by as _,
            event.checked_out_at,
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() == 0 {
            return Err(AppError::NoRowsAffectedError(
                "No checkout record has been created".into(),
            ));
        }

        tx.commit().await.map_err(AppError::TransactionError)?;

        Ok(())
    }

    /// 返却操作を行う
    async fn update_returned(&self, event: UpdateReturned) -> AppResult<()> {
        let mut tx = self.db.begin().await?;

        // トランザクション分離レベルをSERIALIZABLEに設定
        self.set_transaction_serializable(&mut tx).await?;

        // 事前に以下をチェック
        // - 指定の蔵書が存在するか
        // - 指定の貸出が存在するか、かつ借りたユーザーが指定のユーザーと同じか
        {
            let res = sqlx::query_as!(
                CheckoutStateRow,
                r#"
                    SELECT 
                        b.book_id, 
                        c.checkout_id AS "checkout_id?: CheckoutId", 
                        c.user_id AS "user_id?: UserId"
                    FROM books AS b
                    LEFT OUTER JOIN checkouts AS c USING(book_id)
                    WHERE book_id = $1
                "#,
                event.book_id as _
            )
            .fetch_optional(&mut *tx)
            .await
            .map_err(AppError::SpecificOperationError)?;

            match res {
                // 蔵書が存在しない場合
                None => {
                    return Err(AppError::EntityNotFound(format!(
                        "書籍（{}）が見つかりませんでした",
                        event.book_id
                    )));
                }
                // 指定の蔵書が貸出中であり、貸出IDまたは借りたユーザーが異なる場合
                Some(CheckoutStateRow {
                    checkout_id: Some(c),
                    user_id: Some(u),
                    ..
                }) if (c, u) != (event.checkout_id, event.returned_by) => {
                    return Err(AppError::UnprocessableEntity(format!(
                        "指定の貸出（ID（{}）、ユーザー（{}）、書籍（{}））は返却できません",
                        event.checkout_id, event.returned_by, event.book_id
                    )));
                }
                // それ以外は処理を続行
                _ => {}
            }
        }

        // DB上の返却操作として、checkoutsテーブルの該当貸出IDのレコードを
        // returned_atを追加して、returned_checkoutsテーブルにINSERTする
        let res = sqlx::query!(
            r#"
            INSERT INTO returned_checkouts (checkout_id, book_id, user_id, checked_out_at, returned_at)
            SELECT checkout_id, book_id, user_id, checked_out_at, $1
            FROM checkouts
            WHERE checkout_id = $2
            "#,
            event.returned_at,
            event.checkout_id as _,
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() == 0 {
            return Err(AppError::NoRowsAffectedError(
                "No returning record has been updated".into(),
            ));
        }

        // 返却操作が成功したら、checkoutsテーブルから該当の貸出レコードを削除
        let res = sqlx::query!(
            r#"
            DELETE FROM checkouts
            WHERE checkout_id = $1
            "#,
            event.checkout_id as _,
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() == 0 {
            return Err(AppError::NoRowsAffectedError(
                "No checkout record has been deleted".into(),
            ));
        }

        tx.commit().await.map_err(AppError::TransactionError)?;

        Ok(())
    }

    /// 全ての未返却の貸出情報を取得する
    async fn find_unreturned_all(&self) -> AppResult<Vec<Checkout>> {
        sqlx::query_as!(
            CheckoutRow,
            r#"
                SELECT
                    c.checkout_id,
                    c.book_id,
                    c.user_id,
                    c.checked_out_at,
                    b.title,
                    b.author,
                    b.isbn
                FROM checkouts AS c
                INNER JOIN books AS b USING(book_id)
                ORDER BY c.checked_out_at ASC
            "#
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map(|rows| rows.into_iter().map(Checkout::from).collect())
        .map_err(AppError::SpecificOperationError)
    }

    /// ユーザーIDに紐づく未返却の貸出情報を取得する
    async fn find_unreturned_by_user_id(&self, user_id: UserId) -> AppResult<Vec<Checkout>> {
        sqlx::query_as!(
            CheckoutRow,
            r#"
                SELECT
                    c.checkout_id,
                    c.book_id,
                    c.user_id,
                    c.checked_out_at,
                    b.title,
                    b.author,
                    b.isbn
                FROM checkouts AS c
                INNER JOIN books AS b USING(book_id)
                WHERE c.user_id = $1
                ORDER BY c.checked_out_at ASC
            "#,
            user_id as _
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map(|rows| rows.into_iter().map(Checkout::from).collect())
        .map_err(AppError::SpecificOperationError)
    }

    /// 蔵書の貸出履歴（返却済みも含む）を取得する
    async fn find_history_by_book_id(&self, book_id: BookId) -> AppResult<Vec<Checkout>> {
        // 未返却の貸出情報を取得
        let checkout: Option<Checkout> = self.find_unreturned_by_book_id(book_id).await?;

        // 返却済みの貸出情報を取得
        let mut checkout_histories: Vec<Checkout> = sqlx::query_as!(
            ReturnedCheckoutRow,
            r#"
                SELECT
                    rc.checkout_id,
                    rc.book_id,
                    rc.user_id,
                    rc.checked_out_at,
                    rc.returned_at,
                    b.title,
                    b.author,
                    b.isbn
                FROM returned_checkouts AS rc
                INNER JOIN books AS b USING(book_id)
                WHERE rc.book_id = $1
                ORDER BY rc.checked_out_at ASC
            "#,
            book_id as _
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?
        .into_iter()
        .map(Checkout::from)
        .collect();

        // 未返却の貸出情報があれば、履歴の先頭に追加
        if let Some(c) = checkout {
            checkout_histories.insert(0, c);
        }

        Ok(checkout_histories)
    }
}

impl CheckoutRepositoryImpl {
    /// トランザクションの分離レベルをSERIALIZABLEに設定する
    async fn set_transaction_serializable(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> AppResult<()> {
        sqlx::query!("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
            .execute(&mut **tx)
            .await
            .map_err(AppError::SpecificOperationError)?;
        Ok(())
    }

    /// 蔵書IDに紐づく未返却の貸出情報を取得する
    async fn find_unreturned_by_book_id(&self, book_id: BookId) -> AppResult<Option<Checkout>> {
        let res = sqlx::query_as!(
            CheckoutRow,
            r#"
                SELECT
                    c.checkout_id,
                    c.book_id,
                    c.user_id,
                    c.checked_out_at,
                    b.title,
                    b.author,
                    b.isbn
                FROM checkouts AS c
                INNER JOIN books AS b USING(book_id)
                WHERE c.book_id = $1
            "#,
            book_id as _
        )
        .fetch_optional(self.db.inner_ref())
        .await
        .map(|row| row.map(Checkout::from))
        .map_err(AppError::SpecificOperationError)?;

        Ok(res)
    }
}
