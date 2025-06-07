use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct RedeemableSubscription {
    pub sub_id: String,
    pub module: String,
    pub subscriber: String,
    pub recipient: String,
    pub amount: String,
    pub next_redeem_at: i32,
}
