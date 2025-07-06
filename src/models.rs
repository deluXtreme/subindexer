use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct RedeemableSubscription {
    pub id: String,
    pub subscriber: String,
    pub recipient: String,
    pub amount: String,
    pub category: Category,
    pub next_redeem_at: i32,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "integer", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Category {
    #[sqlx(rename = "0")]
    #[serde(rename = "trusted")]
    Trusted = 0,
    #[sqlx(rename = "1")]
    #[serde(rename = "untrusted")]
    Untrusted = 1,
    #[sqlx(rename = "2")]
    #[serde(rename = "group")]
    Group = 2,
}
