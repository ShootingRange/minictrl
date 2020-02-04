extern crate minictrl;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use crate::minictrl::get5::basic;
use minictrl::common::Side;

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
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

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/again", web::get().to(index2))
    })
        .bind("127.0.0.1:8088")?
        .run()
        .await
}
