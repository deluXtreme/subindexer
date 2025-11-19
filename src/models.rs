use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct RedeemableSubscription {
    pub contract_address: String,
    pub id: String,
    pub subscriber: String,
    pub recipient: String,
    pub amount: String,
    pub periods: i32,
    pub category: Category,
    pub next_redeem_at: i32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Subscription {
    pub contract_address: String,
    pub id: String,
    pub subscriber: String,
    pub recipient: String,
    pub amount: String,
    pub category: Category,
    pub frequency: i32,
    pub creation_timestamp: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Trusted,
    Untrusted,
    Group,
}

impl<DB: sqlx::Database> sqlx::Type<DB> for Category
where
    String: sqlx::Type<DB>,
{
    fn type_info() -> <DB as sqlx::Database>::TypeInfo {
        <String as sqlx::Type<DB>>::type_info()
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for Category {
    fn decode(
        value: <sqlx::Sqlite as sqlx::Database>::ValueRef<'r>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let value = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        match value.as_str() {
            "0" => Ok(Category::Trusted),
            "1" => Ok(Category::Untrusted),
            "2" => Ok(Category::Group),
            "trusted" => Ok(Category::Trusted),
            "untrusted" => Ok(Category::Untrusted),
            "group" => Ok(Category::Group),
            _ => Err(format!("invalid category value: {}", value).into()),
        }
    }
}
