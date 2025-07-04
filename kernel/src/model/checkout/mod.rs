use chrono::{DateTime, Utc};

use crate::model::id::{BookId, CheckoutId, UserId};

pub mod event;

#[derive(Debug)]
pub struct Checkout {
    pub id: CheckoutId,
    pub checked_out_by: UserId,
    pub checked_out_at: DateTime<Utc>,
    pub returned_at: Option<DateTime<Utc>>,
    pub book: CheckoutBook,
}

#[derive(Debug)]
pub struct CheckoutBook {
    pub id: BookId,
    pub title: String,
    pub author: String,
    pub isbn: String,
}
