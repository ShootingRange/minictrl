use crate::common::SideType;
use crate::database;
use crate::database::models::Player;
use crate::database::Database;
use crate::get5::basic::{
    Match as Get5Match, Player as Get5Player, Spectators as Get5Spectators, Team as Get5Team,
};
use diesel::result::Error;
use std::convert::Infallible;
use std::sync::Arc;
use warp::http::StatusCode;

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

pub async fn handler_get5_config(
    id: i32,
    db: Arc<Database>,
) -> Result<Box<dyn warp::reply::Reply>, Infallible> {
    let error_formatter = |err| {
        let reply = match err {
            database::Error::DB(err) => match err {
                Error::NotFound => {
                    trace!("Not Found: {}", err);
                    StatusCode::NOT_FOUND
                }
                _ => {
                    error!("Database Error: {}", err);
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            },
            database::Error::Pool(err) => {
                error!("Pool Error: {}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        Ok(Box::new(reply) as Box<dyn warp::reply::Reply>)
    };

    // Match
    let r#match = match db.get_match(id) {
        Ok(m) => match m {
            None => {
                return Ok(Box::new(StatusCode::NOT_FOUND) as Box<dyn warp::reply::Reply>);
            }
            Some(m) => m,
        },
        Err(err) => {
            return error_formatter(err);
        }
    };

    // Teams
    let team1 = match db.get_team(r#match.team1_id) {
        Ok(team) => match team {
            None => {
                error!("match (id={}) referenced team (id={}) in the database, but no such team exists", r#match.id, r#match.team1_id);
                return Ok(
                    Box::new(StatusCode::INTERNAL_SERVER_ERROR) as Box<dyn warp::reply::Reply>
                );
            }
            Some(team) => team,
        },
        Err(err) => return error_formatter(err),
    };
    let team2 = match db.get_team(r#match.team2_id) {
        Ok(team) => match team {
            None => {
                error!("match (id={}) referenced team (id={}) in the database, but no such team exists", r#match.id, r#match.team2_id);
                return Ok(
                    Box::new(StatusCode::INTERNAL_SERVER_ERROR) as Box<dyn warp::reply::Reply>
                );
            }
            Some(team) => team,
        },
        Err(err) => return error_formatter(err),
    };

    // Players
    let team1_players = match db.get_team_players(r#match.team1_id) {
        Ok(players) => match players {
            None => {
                error!("Match (id={}) referenced Team (id={}) in the database, but no such Team exists", r#match.id, r#match.team1_id);
                return Ok(
                    Box::new(StatusCode::INTERNAL_SERVER_ERROR) as Box<dyn warp::reply::Reply>
                );
            }
            Some(players) => players.iter().filter_map(format_player).collect(),
        },
        Err(err) => return error_formatter(err),
    };
    let team2_players = match db.get_team_players(r#match.team2_id) {
        Ok(players) => match players {
            None => {
                error!("Match (id={}) referenced Team (id={}) in the database, but no such Team exists", r#match.id, r#match.team2_id);
                return Ok(
                    Box::new(StatusCode::INTERNAL_SERVER_ERROR) as Box<dyn warp::reply::Reply>
                );
            }
            Some(players) => players.iter().filter_map(format_player).collect(),
        },
        Err(err) => return error_formatter(err),
    };

    // Spectators
    let spectators = match db.get_spectators(r#match.id) {
        Ok(spectators) => match spectators {
            None => {
                error!("no Match with id {} exists", r#match.id);
                return Ok(
                    Box::new(StatusCode::INTERNAL_SERVER_ERROR) as Box<dyn warp::reply::Reply>
                );
            }
            Some(spectators) => {
                if !spectators.is_empty() {
                    // Format the spectators SteamID
                    Some(Get5Spectators {
                        name: "Spectators".to_string(),
                        players: spectators
                            .iter()
                            .map(|steamid| Get5Player {
                                steamID: steamid.clone(),
                                name: None,
                            })
                            .collect(),
                    })
                } else {
                    // Get5 breaks on empty lists so omit the field if there is no spectators
                    None
                }
            }
        },
        Err(err) => return error_formatter(err),
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
        spectators,
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

    Ok(Box::new(warp::reply::json(&get5_match)) as Box<dyn warp::reply::Reply>)
}
