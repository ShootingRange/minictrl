extern crate minictrl;

use minictrl::database::run_migrations;
use minictrl::web::webserver_start;
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{error, trace};

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    // Setup logging
    tracing_subscriber::fmt()
        // Configure formatting settings.
        .with_target(false)
        .with_timer(tracing_subscriber::fmt::time::time())
        .with_level(true)
        // Set the collector as the default.
        .init();

    // Establish database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(std::env::var("DATABASE_URL")?.as_str())
        .await?;

    run_migrations(&pool).await;

    webserver_start(pool).await
}
