use crate::actors::database::player::FindPlayersByTeamId;
use crate::actors::database::r#match::FindMatchById;
use crate::actors::database::team::FindTeamById;
use crate::actors::database::DbExecutor;
use crate::common::SideType;
use crate::database::models::{Player, Team};
use crate::get5::basic::{Match as Get5Match, Player as Get5Player, Team as Get5Team};
use actix::Addr;
use actix_web::error::InternalError;
use actix_web::web;
use actix_web::{HttpRequest, Responder};

pub mod graphql;

async fn get_team(id: i32, db: Addr<DbExecutor>) -> Result<Team, InternalError<String>> {
    let team = db.send(FindTeamById { id }).await;

    match team {
        Ok(t) => match t {
            Ok(t) => Ok(t),
            Err(_err) => Err(InternalError::new(
                "Database error".to_string(),
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            )),
        },
        Err(_err) => Err(InternalError::new(
            "Mailbox error".to_string(),
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

async fn get_players(
    team_id: i32,
    db: Addr<DbExecutor>,
) -> Result<Vec<Player>, InternalError<String>> {
    let players = db.send(FindPlayersByTeamId { team_id }).await;

    match players {
        Ok(t) => match t {
            Ok(players) => Ok(players),
            Err(_err) => Err(actix_web::error::InternalError::new(
                "Database error".to_string(),
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            )),
        },
        Err(_err) => Err(actix_web::error::InternalError::new(
            "Mailbox error".to_string(),
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

fn format_player(player: &Player) -> Option<Get5Player> {
    if let Some(steamid) = player.steamid.clone() {
        Some(Get5Player {
            steamID: steamid,
            name: Some(player.name.clone()), // TODO make player name optional in database
        })
    } else {
        None
    }
}

pub async fn get5_config(db: web::Data<Addr<DbExecutor>>, req: HttpRequest) -> impl Responder {
    let id = if let Some(id) = req.match_info().get("id") {
        id
    } else {
        return Err(actix_web::error::InternalError::new(
            "Missing ID".to_string(),
            actix_web::http::StatusCode::BAD_REQUEST,
        ));
    };

    let id = if let Ok(id) = id.parse::<i32>() {
        id
    } else {
        return Err(actix_web::error::InternalError::new(
            "Invalid ID".to_string(),
            actix_web::http::StatusCode::BAD_REQUEST,
        ));
    };

    let r#match = db.send(FindMatchById { id }).await;

    let r#match = match r#match {
        Ok(m) => match m {
            Ok(m) => m,
            Err(_err) => {
                return Err(actix_web::error::InternalError::new(
                    "Database error".to_string(),
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                ))
            }
        },
        Err(_err) => {
            return Err(actix_web::error::InternalError::new(
                "Mailbox error".to_string(),
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    };

    let db = db.get_ref().clone();

    let team1 = match get_team(r#match.team1_id, db.clone()).await {
        Ok(team) => team,
        Err(err) => return Err(err),
    };
    let team2 = match get_team(r#match.team2_id, db.clone()).await {
        Ok(team) => team,
        Err(err) => return Err(err),
    };

    let team1_players = match get_players(r#match.team1_id, db.clone()).await {
        Ok(players) => players.iter().filter_map(format_player).collect(),
        Err(err) => return Err(err),
    };
    let team2_players = match get_players(r#match.team2_id, db.clone()).await {
        Ok(players) => players.iter().filter_map(format_player).collect(),
        Err(err) => return Err(err),
    };

    let r#match = db.send(FindMatchById { id }).await;

    let r#match = match r#match {
        Ok(m) => match m {
            Ok(m) => m,
            Err(_err) => {
                return Err(actix_web::error::InternalError::new(
                    "Database error".to_string(),
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                ))
            }
        },
        Err(_err) => {
            return Err(actix_web::error::InternalError::new(
                "Mailbox error".to_string(),
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    };

    let get5_match = Get5Match {
        matchid: Some(r#match.id.to_string()),
        num_maps: Some(r#match.num_maps),
        maplist: Some(vec![
            "de_dust2".to_string(),
            "de_cbble".to_string(),
            "de_train".to_string(),
        ]), // TODO
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
            flag: team1.country,
            logo: team1.logo,
            players: team1_players,
            series_score: None,
            match_text: None,
        },
        team2: Get5Team {
            name: team2.name,
            tag: None,
            flag: team2.country,
            logo: team2.logo,
            players: team2_players,
            series_score: None,
            match_text: None,
        },
        match_title: None,
    };

    Result::Ok(actix_web::web::Json(get5_match))
}
