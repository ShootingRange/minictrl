extern crate minictrl;
extern crate dotenv;

use actix_web::{web, App, HttpServer, Responder};
use crate::minictrl::get5::basic;
use minictrl::common::Side;
use actix::SyncArbiter;
use diesel::{PgConnection, Connection};
use crate::minictrl::actors::database::*;
use dotenv::dotenv;
use std::env;
use minictrl::web::graphql::*;
use actix_cors::Cors;

async fn index2() -> impl Responder {
    let m = basic::Match {
        matchid: Some("foo".to_string()),
        num_maps: None,
        maplist: None,
        skip_veto: None,
        side_type: Some(Side::AlwaysKnife),
        players_per_team: None,
        min_players_to_ready: None,
        favored_percentage_team1: None,
        favored_percentage_text: None,
        cvars: None,
        spectators: None,
        team1: basic::Team {
            name: "".to_string(),
            tag: None,
            flag: None,
            logo: None,
            players: vec![],
            series_score: None,
            match_text: None
        },
        team2: basic::Team {
            name: "".to_string(),
            tag: None,
            flag: None,
            logo: None,
            players: vec![],
            series_score: None,
            match_text: None
        },
        match_title: None
    };
    web::Json(m)
    //HttpResponse::Ok().body("Hello world again!")
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    // Start 3 parallel db executors
    let db_addr = SyncArbiter::start(3, move || {
        DbExecutor{
            conn: PgConnection::establish(database_url.as_str()).unwrap()
        }
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
                    .finish()
            )
            .data(db_addr.clone())
            .data(create_schema())
            //.service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/again").route(web::get().to(index2)))
            //.service(web::resource("/teams").route(web::get().to(list_teams)))
            .service(web::resource("/graphql").route(web::post().to(graphql)))
            .service(web::resource("/graphiql").route(web::get().to(graphiql)))
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
