extern crate minictrl;
extern crate dotenv;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use crate::minictrl::get5::basic;
use minictrl::common::Side;
use actix::{SyncArbiter, Addr};
use diesel::{PgConnection, Connection};
use crate::minictrl::actors::database::*;
use dotenv::dotenv;
use std::env;
use minictrl::database::models::NewTeam;
use minictrl::web::graphql::*;

async fn index(data: web::Data<State>) -> impl Responder {
    let actor_resp = data.db
        .send(NewTeam {
            name: "foo".to_string(),
            country: None,
            logo: None
        })
        .await;

    match actor_resp {
        Ok(db_resp) => {
            match db_resp {
                Ok(_) => {
                    HttpResponse::Ok().body("success")
                },
                Err(_) => {
                    HttpResponse::InternalServerError().body("Database error")
                },
            }
        },
        Err(_) => {
            HttpResponse::InternalServerError().body("Actor mailbox error")
        },
    }
}

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

async fn list_teams(data: web::Data<State>) -> impl Responder {
    let actor_resp = data.db
        .send(ListTeams {})
        .await;

    match actor_resp {
        Ok(db_resp) => {
            match db_resp {
                Ok(teams) => {
                    match serde_json::to_string(&teams) {
                        Ok(json) => {
                            HttpResponse::Ok().body(&json)
                        },
                        Err(_) => {
                            HttpResponse::InternalServerError().body("failed to serialize teams")
                        },
                    }
                },
                Err(_) => {
                    HttpResponse::InternalServerError().body("Database error")
                },
            }
        },
        Err(_) => {
            HttpResponse::InternalServerError().body("Actor mailbox error")
        },
    }
}

/// This is state where we will store *DbExecutor* address.
struct State {
    db: Addr<DbExecutor>,
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    // Start 3 parallel db executors
    let addr = SyncArbiter::start(3, move || {
        DbExecutor{
            conn: PgConnection::establish(database_url.as_str()).unwrap()
        }
    });

    let schema = std::sync::Arc::new(create_schema());

    // Start http server
    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .data(State { db: addr.clone() })
            .data(schema.clone())
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/again").route(web::get().to(index2)))
            .service(web::resource("/teams").route(web::get().to(list_teams)))
            .service(web::resource("/graphql").route(web::post().to(graphql)))
            .service(web::resource("/graphiql").route(web::get().to(graphiql)))
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
