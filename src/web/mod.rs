use crate::database::Database;
use slog::Logger;
use std::convert::Infallible;
use std::sync::Arc;
use warp::Filter;

fn with_logger(logger: Logger) -> impl Filter<Extract = (Logger,), Error = Infallible> + Clone {
    warp::any().map(move || logger.clone())
}

fn with_db(
    db: Arc<Database>,
) -> impl Filter<Extract = (Arc<Database>,), Error = Infallible> + Clone {
    warp::any().map(move || db.clone())
}

pub fn router(
    logger: Logger,
    db: Arc<Database>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let get5_config = warp::path("get5")
        .and(warp::path("config"))
        .and(warp::path::param::<i32>())
        .and(warp::get())
        .and(warp::path::end())
        .and(with_logger(logger))
        .and(with_db(db))
        .and_then(get5::handler_get5_config);

    let router = get5_config;

    router
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

pub async fn handler_get5_config(
    id: i32,
    logger: Logger,
    db: Arc<Database>,
) -> Result<Box<dyn warp::reply::Reply>, Infallible> {
    let error_formatter = |err| {
        let reply = match err {
            database::Error::DB(err) => match err {
                Error::NotFound => {
                    trace!(logger, "Not Found: {}", err);
                    StatusCode::NOT_FOUND
                }
                _ => {
                    error!(logger, "Database Error: {}", err);
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            },
            database::Error::Pool(err) => {
                error!(logger, "Pool Error: {}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        Ok(Box::new(reply) as Box<dyn warp::reply::Reply>)
    };

    let r#match = db.find_match_by_id(id);

    let r#match = match r#match {
        Ok(m) => m,
        Err(err) => {
            return error_formatter(err);
        }
    };

    let team1 = match db.find_team_by_id(r#match.team1_id) {
        Ok(team) => team,
        Err(err) => return error_formatter(err),
    };
    let team2 = match db.find_team_by_id(r#match.team2_id) {
        Ok(team) => team,
        Err(err) => return error_formatter(err),
    };

    let team1_players = match db.get_team_players(r#match.team1_id) {
        Ok(players) => players.iter().filter_map(format_player).collect(),
        Err(err) => return error_formatter(err),
    };
    let team2_players = match db.get_team_players(r#match.team2_id) {
        Ok(players) => players.iter().filter_map(format_player).collect(),
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

    Ok(Box::new(warp::reply::json(&get5_match)) as Box<dyn warp::reply::Reply>)
}
