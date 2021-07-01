use sqlx::Acquire;
use sqlx::Postgres;
use tide::{Body, Response, StatusCode};
use tide_sqlx::SQLxRequestExt;

use crate::common::SideType;
use crate::database::models::Player;
use crate::database::*;
use crate::get5::basic::{
    Match as Get5Match, Player as Get5Player, Spectators as Get5Spectators, Team as Get5Team,
};
use crate::web::State;

fn format_player(player: &Player) -> Option<Get5Player> {
    Some(Get5Player {
        steamID: player.steamid.clone(),
        name: player.name.clone(),
    })
}

#[derive(Deserialize, Debug)]
struct MatchIdArgs {
    id: i32,
}

pub async fn endpoint_get5_config(req: tide::Request<State>) -> tide::Result<Response> {
    let mut pool = req.sqlx_conn::<Postgres>().await;
    let mut db_conn = pool.acquire().await?;

    let id = req.query::<MatchIdArgs>()?.id;

    // Match
    let r#match = match get_match(&mut db_conn, id) {
        Ok(m) => match m {
            None => {
                return tide::Result::Ok(Response::new(StatusCode::NotFound));
            }
            Some(m) => m,
        },
        Err(err) => {
            return tide::Result::Err(tide::Error::new(StatusCode::InternalServerError, err));
        }
    };

    // Teams
    let team1 = match get_team(&mut db_conn, r#match.team1_id) {
        Ok(team) => match team {
            None => {
                error!("match (id={}) referenced team (id={}) in the database, but no such team exists", r#match.id, r#match.team1_id);
                return tide::Result::Err(tide::Error::new(
                    StatusCode::InternalServerError,
                    anyhow::Error::msg(""),
                ));
            }
            Some(team) => team,
        },
        Err(err) => {
            return tide::Result::Err(tide::Error::new(StatusCode::InternalServerError, err))
        }
    };
    let team2 = match get_team(&mut db_conn, r#match.team2_id) {
        Ok(team) => match team {
            None => {
                error!("match (id={}) referenced team (id={}) in the database, but no such team exists", r#match.id, r#match.team2_id);
                return tide::Result::Err(tide::Error::new(
                    StatusCode::InternalServerError,
                    anyhow::Error::msg(""),
                ));
            }
            Some(team) => team,
        },
        Err(err) => {
            return tide::Result::Err(tide::Error::new(StatusCode::InternalServerError, err))
        }
    };

    // Players
    let team1_players = match get_team_players(&mut db_conn, r#match.team1_id) {
        Ok(players) => match players {
            None => {
                error!("Match (id={}) referenced Team (id={}) in the database, but no such Team exists", r#match.id, r#match.team1_id);
                return tide::Result::Err(tide::Error::new(
                    StatusCode::InternalServerError,
                    anyhow::Error::msg(""),
                ));
            }
            Some(players) => players.iter().filter_map(format_player).collect(),
        },
        Err(err) => {
            return tide::Result::Err(tide::Error::new(StatusCode::InternalServerError, err))
        }
    };
    let team2_players = match get_team_players(&mut db_conn, r#match.team2_id) {
        Ok(players) => match players {
            None => {
                error!("Match (id={}) referenced Team (id={}) in the database, but no such Team exists", r#match.id, r#match.team2_id);
                return tide::Result::Err(tide::Error::new(
                    StatusCode::InternalServerError,
                    anyhow::Error::msg(""),
                ));
            }
            Some(players) => players.iter().filter_map(format_player).collect(),
        },
        Err(err) => {
            return tide::Result::Err(tide::Error::new(StatusCode::InternalServerError, err))
        }
    };

    // Spectators
    let spectators = match get_spectators(&mut db_conn, r#match.id) {
        Ok(spectators) => match spectators {
            None => {
                error!("no Match with id {} exists", r#match.id);
                return tide::Result::Err(tide::Error::new(
                    StatusCode::InternalServerError,
                    anyhow::Error::msg(""),
                ));
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
        Err(err) => {
            return tide::Result::Err(tide::Error::new(StatusCode::InternalServerError, err))
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

    let mut resp = Response::new(StatusCode::NotFound);
    resp.set_body(Body::from_json(&get5_match)?);
    Ok(resp)
}
