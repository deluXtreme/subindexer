use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct RedeemableSubscription {
    pub id: String,
    pub subscriber: String,
    pub recipient: String,
    pub amount: String,
    pub trusted: bool,
    pub next_redeem_at: i32,
}
