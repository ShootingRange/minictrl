extern crate dotenv;
extern crate minictrl;

use crate::minictrl::actors::database::*;
use actix::SyncArbiter;
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use diesel::{Connection, PgConnection};
use dotenv::dotenv;
use minictrl::web::graphql::*;
use std::env;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Start 3 parallel db executors
    let db_addr = SyncArbiter::start(3, move || DbExecutor {
        conn: PgConnection::establish(database_url.as_str()).unwrap(),
    });

    println!("http://localhost:8080/graphiql");

    // Start http server
    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .wrap(
                Cors::new()
                    .allowed_origin("http://localhost:8080")
                    .supports_credentials()
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
    .await
}
