//pub use crate::csgo::receiver::http::handler_log_receiver;
//pub use crate::get5::handler_get5_config;
use crate::web::get5::endpoint_get5_config;
use sqlx::{Pool, Postgres};
use tide_sqlx::SQLxMiddleware;
use tide_tracing::TraceMiddleware;

pub type State = ();

mod get5;

pub async fn webserver_start(db_pool: Pool<Postgres>) -> anyhow::Result<()> {
    // Setup http server
    let mut app = tide::new();
    app.with(SQLxMiddleware::from(db_pool));
    app.with(TraceMiddleware::new());

    // TODO setup routes
    app.at("/")
        .get(|req: tide::Request<()>| async move { Ok("hello world") });

    app.at("/api/get5/config").get(endpoint_get5_config);

    // Start http server
    app.listen("127.0.0.1:8080").await?;

    Ok(())
}
