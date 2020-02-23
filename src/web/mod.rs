use crate::actors::database::r#match::FindMatchById;
use crate::actors::database::team::FindTeamById;
use crate::actors::database::DbExecutor;
use crate::get5::basic::{Match as Get5Match, Team as Get5Team};
use actix::Addr;
use actix_web::{Responder, HttpRequest};
use actix_web::web;
use crate::common::SideType;

pub mod graphql;

pub async fn get5_config(
    db: web::Data<Addr<DbExecutor>>,
    req: HttpRequest,
) -> impl Responder {
    let id = if let Some(id) = req.match_info().get("id") {
        id
    } else {
        return Err(actix_web::error::InternalError::new(
            "Missing ID",
            actix_web::http::StatusCode::BAD_REQUEST,
        ))
    };

    let id = if let Ok(id) = id.parse::<i32>() {
        id
    } else {
        return Err(actix_web::error::InternalError::new(
            "Invalid ID",
            actix_web::http::StatusCode::BAD_REQUEST,
        ))
    };

    let r#match = db.send(FindMatchById { id }).await;

    let r#match = match r#match {
        Ok(m) => match m {
            Ok(m) => m,
            Err(err) => {
                println!("{}: {}", id, err);
                return Err(actix_web::error::InternalError::new(
                    "Database error",
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                ))
            }
        },
        Err(_err) => {
            return Err(actix_web::error::InternalError::new(
                "Mailbox error",
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    };

    let team1 = db
        .send(FindTeamById {
            id: r#match.team1_id,
        })
        .await;

    let team1 = match team1 {
        Ok(t) => match t {
            Ok(t) => t,
            Err(_err) => {
                return Err(actix_web::error::InternalError::new(
                    "Database error",
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                ))
            }
        },
        Err(_err) => {
            return Err(actix_web::error::InternalError::new(
                "Mailbox error",
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    };

    let team2 = db
        .send(FindTeamById {
            id: r#match.team1_id,
        })
        .await;

    let team2 = match team2 {
        Ok(t) => match t {
            Ok(t) => t,
            Err(_err) => {
                return Err(actix_web::error::InternalError::new(
                    "Database error",
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                ))
            }
        },
        Err(_err) => {
            return Err(actix_web::error::InternalError::new(
                "Mailbox error",
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    };

    // TODO players

    let get5_match = Get5Match {
        matchid: Some(r#match.id.to_string()),
        num_maps: Some(r#match.num_maps),
        maplist: Some(vec!["de_dust2".to_string(), "de_cbble".to_string(), "de_train".to_string()]), // TODO
        skip_veto: Some(r#match.skip_veto),
        side_type: Some(SideType::Standard), // TODO
        players_per_team: Some(r#match.players_per_team),
        min_players_to_ready: Some(r#match.min_player_to_ready),
        favored_percentage_team1: None,
        favored_percentage_text: None,
        cvars: None,
        spectators: None, //TODO
        team1: Get5Team {
            name: team1.name,
            tag: None,
            flag: None, // TODO
            logo: team1.logo,
            players: vec![], // TODO
            series_score: None,
            match_text: None,
        },
        team2: Get5Team {
            name: team2.name,
            tag: None,
            flag: None, // TODO
            logo: team2.logo,
            players: vec![], // TODO
            series_score: None,
            match_text: None,
        },
        match_title: None,
    };

    Result::Ok(actix_web::web::Json(get5_match))
}
