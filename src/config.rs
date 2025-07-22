use std::{env, str::FromStr};

use alloy::signers::local::PrivateKeySigner;
use sqlx::{PgPool, postgres::PgPoolOptions};

pub struct Config {
    pub pool: PgPool,
    pub redeemer: PrivateKeySigner,
    // Optional overrides:
    pub api_port: u16,
    pub rpc_url: String,
    pub redeem_interval: u64,
}

impl Config {
    pub async fn from_env() -> Self {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(&database_url)
            .await
            .expect("Failed to connect to Postgres");

        Self {
            pool,
            redeemer: PrivateKeySigner::from_str(&env::var("REDEEMER_PK").expect("REDEEMER_PK must be set"))
                .expect("Failed to parse REDEEMER_PK"),
            api_port: env::var("API_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .expect("API_PORT must be a number"),
            rpc_url: env::var("GNOSIS_RPC_URL")
                .unwrap_or_else(|_| "https://rpc.gnosischain.com/".to_string()),
            redeem_interval: env::var("REDEEM_INTERVAL")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .expect("REDEEM_INTERVAL must be a number"),
        }
    }

    pub fn log_non_secrets(&self) {
        tracing::info!(
            "Config: api_port={}, redeem_interval={}, rpc_url={}, redeemer={}",
            self.api_port,
            self.redeem_interval,
            self.rpc_url,
            self.redeemer.address()
        );
    }
}
