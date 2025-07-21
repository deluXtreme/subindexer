use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct RedeemableSubscription {
    #[serde(serialize_with = "hex::serialize")]
    pub id: Vec<u8>,
    pub subscriber: String,
    pub recipient: String,
    pub amount: String,
    pub periods: i32,
    pub category: Category,
    pub next_redeem_at: i32,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[repr(i16)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Trusted = 0,
    Untrusted = 1,
    Group = 2,
}

mod hex {
    use serde::Serializer;

    pub fn serialize<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("0x{}", hex::encode(bytes)))
    }
}
