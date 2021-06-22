extern crate dotenv;
extern crate minictrl;

use diesel::r2d2;
use diesel::r2d2::event::{AcquireEvent, CheckinEvent, CheckoutEvent, ReleaseEvent, TimeoutEvent};
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use dotenv::dotenv;
use minictrl::database::Database;
use std::env;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{error, trace};

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Setup logging
    tracing_subscriber::fmt()
        // Configure formatting settings.
        .with_target(false)
        .with_timer(tracing_subscriber::fmt::time::time())
        .with_level(true)
        // Set the collector as the default.
        .init();

    // Setup database connection
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Establish database connection pool
    let db_manager: ConnectionManager<PgConnection> = r2d2::ConnectionManager::new(database_url);
    let db_conn = r2d2::Builder::new()
        .error_handler(Box::new(R2D2StructuredLoggingHandler {}))
        .event_handler(Box::new(R2D2StructuredLoggingHandler {}))
        .build(db_manager)
        .expect("failed to connect to database");

    let db = Arc::new(Database::new(db_conn));

    // Start http server
    let listen_addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    let router = minictrl::web::router(db);
    warp::serve(router).run(listen_addr).await;
}

#[derive(Debug)]
struct R2D2StructuredLoggingHandler {}

impl<E: Debug> r2d2::HandleError<E> for R2D2StructuredLoggingHandler {
    fn handle_error(&self, error: E) {
        error!("{:?}", error);
    }
}

impl r2d2::HandleEvent for R2D2StructuredLoggingHandler {
    fn handle_acquire(&self, event: AcquireEvent) {
        trace!("{:?}", event);
    }

    fn handle_release(&self, event: ReleaseEvent) {
        trace!("{:?}", event);
    }

    fn handle_checkout(&self, event: CheckoutEvent) {
        trace!("{:?}", event);
    }

    fn handle_timeout(&self, event: TimeoutEvent) {
        trace!("{:?}", event);
    }

    fn handle_checkin(&self, event: CheckinEvent) {
        trace!("{:?}", event);
    }
}
