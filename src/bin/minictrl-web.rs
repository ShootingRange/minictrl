extern crate dotenv;
extern crate minictrl;

use diesel::r2d2;
use diesel::r2d2::event::{AcquireEvent, CheckinEvent, CheckoutEvent, ReleaseEvent, TimeoutEvent};
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use dotenv::dotenv;
use minictrl::database::Database;
use slog::{error, trace, Logger};
use sloggers::terminal::{Destination, TerminalLoggerBuilder};
use sloggers::types::Severity;
use sloggers::Build;
use std::env;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Setup logging
    let logger = TerminalLoggerBuilder::new()
        .level(Severity::Trace)
        .destination(Destination::Stdout)
        .build()
        .unwrap();

    // Setup database connection
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Establish database connection pool
    let db_manager: ConnectionManager<PgConnection> = r2d2::ConnectionManager::new(database_url);
    let db_conn = r2d2::Builder::new()
        .error_handler(Box::new(R2D2StructuredLoggingHandler {
            logger: logger.clone(),
        }))
        .event_handler(Box::new(R2D2StructuredLoggingHandler {
            logger: logger.clone(),
        }))
        .build(db_manager)
        .expect("failed to connect to database");

    let db = Arc::new(Database::new(db_conn));

    // Start http server
    let listen_addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    trace!(logger, "{}", "http://localhost:8080/graphiql");
    let router = minictrl::web::router(logger, db);
    warp::serve(router).run(listen_addr).await;

    /*HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .wrap(
                Cors::new()
                    .max_age(3600)
                    .finish(),
            )
            .data(db_addr.clone())
            .data(create_schema())
            .service(
                web::resource("/get5/config/{id}").route(web::get().to(minictrl::web::get5_config)),
            )
            .service(web::resource("/graphql").route(web::post().to(graphql)))
            .service(web::resource("/graphiql").route(web::get().to(graphiql)))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await*/
}

#[derive(Debug)]
struct R2D2StructuredLoggingHandler {
    logger: Logger,
}

impl<E: Debug> r2d2::HandleError<E> for R2D2StructuredLoggingHandler {
    fn handle_error(&self, error: E) {
        error!(self.logger, "{:?}", error);
    }
}

impl r2d2::HandleEvent for R2D2StructuredLoggingHandler {
    fn handle_acquire(&self, event: AcquireEvent) {
        trace!(self.logger, "{:?}", event);
    }

    fn handle_release(&self, event: ReleaseEvent) {
        trace!(self.logger, "{:?}", event);
    }

    fn handle_checkout(&self, event: CheckoutEvent) {
        trace!(self.logger, "{:?}", event);
    }

    fn handle_timeout(&self, event: TimeoutEvent) {
        trace!(self.logger, "{:?}", event);
    }

    fn handle_checkin(&self, event: CheckinEvent) {
        trace!(self.logger, "{:?}", event);
    }
}
