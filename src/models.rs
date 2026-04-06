use alloy::primitives::{Address, B256};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedeemableSubscription {
    pub contract_address: Address,
    pub id: B256,
    pub recipient: Address,
    pub subscriber: Address,
    pub amount: String,
    pub periods: i32,
    pub category: Category,
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for RedeemableSubscription {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> sqlx::Result<Self> {
        use sqlx::Row;
        let contract_address = Address::from_str(row.try_get::<&str, _>("contract_address")?)
            .map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
        let id = B256::from_str(row.try_get::<&str, _>("id")?)
            .map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
        let recipient = Address::from_str(row.try_get::<&str, _>("recipient")?)
            .map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
        let subscriber = Address::from_str(row.try_get::<&str, _>("subscriber")?)
            .map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
        Ok(Self {
            contract_address,
            id,
            recipient,
            subscriber,
            amount: row.try_get("amount")?,
            periods: row.try_get("periods")?,
            category: row.try_get("category")?,
        })
    }
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
