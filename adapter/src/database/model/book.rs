use kernel::model::{
    book::{Book, Checkout},
    id::{BookId, CheckoutId, UserId},
    user::{BookOwner, CheckoutUser},
};

use chrono::{DateTime, Utc};

pub struct BookRow {
    pub book_id: BookId,
    pub title: String,
    pub author: String,
    pub isbn: String,
    pub description: String,
    pub owned_by: UserId,
    pub owner_name: String,
}

impl BookRow {
    pub fn into_book(self, checkout: Option<Checkout>) -> Book {
        let BookRow {
            book_id,
            title,
            author,
            isbn,
            description,
            owned_by,
            owner_name,
        } = self;
        Book {
            id: book_id,
            title,
            author,
            isbn,
            description,
            owner: BookOwner {
                id: owned_by,
                name: owner_name,
            },
            checkout,
        }
    }
}

/// ページネーション用のadapter内部の型
pub struct PaginatedBookRow {
    pub total: i64,
    pub id: BookId,
}

/// 貸出情報を格納する型
pub struct BookCheckoutRow {
    pub checkout_id: CheckoutId,
    pub book_id: BookId,
    pub user_id: UserId,
    pub user_name: String,
    pub checked_out_at: DateTime<Utc>,
}

/// Checkout型に変換するFromトレイトの実装
impl From<BookCheckoutRow> for Checkout {
    fn from(value: BookCheckoutRow) -> Self {
        let BookCheckoutRow {
            checkout_id,
            user_id,
            user_name,
            checked_out_at,
            ..
        } = value;

        Checkout {
            checkout_id,
            checked_out_by: CheckoutUser {
                id: user_id,
                name: user_name,
            },
            checked_out_at,
        }
    }
}
