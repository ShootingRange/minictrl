use std::net::IpAddr;

use sqlx::migrate::Migrator;
use sqlx::types::Uuid;
use sqlx::PgConnection;
use sqlx::{Pool, Postgres};

use crate::common::SideType;
use crate::database::models::{CountryCode, MapList, Match, Player, Server, Spectator, Team};

pub mod models;

static MIGRATOR: Migrator = sqlx::migrate!();

pub async fn run_migrations(db_pool: &Pool<Postgres>) -> anyhow::Result<()> {
    MIGRATOR.run(db_pool).await?;

    Ok(())
}

// Match

pub fn set_map_list_for_match(db: &mut PgConnection, match_id: Uuid, maps: Vec<String>) {
    todo!()
}

pub fn get_match_map_list(
    db: &mut PgConnection,
    match_id: Uuid,
) -> Result<Option<Vec<String>>, Error> {
    todo!()
}

pub fn create_match(
    db: &mut PgConnection,
    server_id: Uuid,
    team1_id: Uuid,
    team2_id: Uuid,
    team1_score: Option<i32>,
    team2_score: Option<i32>,
    num_maps: i32,
    skip_veto: bool,
    veto_first: SideType,
    players_per_team: i32,
    min_player_to_ready: i32,
) -> Result<(), Error> {
    todo!()
}

pub async fn get_match(db: &mut PgConnection, match_id: Uuid) -> Result<Option<Match>, Error> {
    let query: sqlx::Result<Match> = sqlx::query_as!(
        Match,
        "SELECT id, server_id, team1_id, team2_id, team1_score, team2_score, num_maps, skip_veto, veto_first AS \"veto_first: SideType\", players_per_team, min_player_to_ready FROM matches WHERE id = $1",
        match_id
    )
    .fetch_one(db)
    .await;

    match query {
        Ok(matsh) => Ok(Some(matsh)),
        Err(err) => match err {
            sqlx::Error::RowNotFound => Ok(None),
            _ => Err(err.into()),
        },
    }
}

pub fn update_match(
    db: &mut PgConnection,
    match_id: Uuid,
    server_id: Uuid,
    team1_id: Uuid,
    team2_id: Uuid,
    team1_score: Option<i32>,
    team2_score: Option<i32>,
    num_maps: i32,
    skip_veto: bool,
    veto_first: SideType,
    players_per_team: i32,
    min_player_to_ready: i32,
) -> Result<(), Error> {
    todo!()
}

pub fn delete_match(db: &mut PgConnection, match_id: Uuid) -> Result<(), Error> {
    todo!()
}

pub fn add_spectator_to_match(
    db: &mut PgConnection,
    spectator_id: Uuid,
    match_id: Uuid,
) -> Result<(), Error> {
    todo!()
}

pub fn remove_spectator_from_match(
    db: &mut PgConnection,
    spectator_id: Uuid,
    match_id: Uuid,
) -> Result<(), Error> {
    todo!()
}

// Player

// TODO index the player by steamid
pub fn add_player_to_team(
    db: &mut PgConnection,
    team_id: Uuid,
    name: String,
    tag: Option<String>,
    steamid: Option<String>,
) -> Result<Uuid, Error> {
    todo!()
}

pub fn remove_player_from_team(db: &mut PgConnection, player_id: Uuid) -> Result<(), Error> {
    todo!()
}

pub fn get_player(db: &mut PgConnection, player_id: Uuid) -> Result<(), Error> {
    todo!()
}

pub fn update_player(db: &mut PgConnection, player_id: Uuid) -> Result<(), Error> {
    todo!()
}

pub async fn get_team_players(
    db: &mut PgConnection,
    team_id: Uuid,
) -> Result<Option<Vec<Player>>, Error> {
    let query: sqlx::Result<Vec<Player>> =
        sqlx::query_as!(Player, "SELECT * FROM players WHERE team_id = $1", team_id)
            .fetch_all(db)
            .await;

    match query {
        Ok(players) => Ok(Some(players)),
        Err(err) => match err {
            sqlx::Error::RowNotFound => Ok(None),
            _ => Err(err.into()),
        },
    }
}

// Server

pub fn get_server(db: &mut PgConnection, server_id: Uuid) -> Result<Option<Server>, Error> {
    todo!()
}

pub fn add_server(
    db: &mut PgConnection,
    host: IpAddr,
    port: u16,
    r#type: Option<String>,
) -> Result<(), Error> {
    todo!()
}

pub fn remove_server(db: &mut PgConnection, server_id: Uuid) -> Result<(), Error> {
    todo!()
}

pub fn server_info(db: &mut PgConnection, server_id: Uuid) -> Result<Option<Server>, Error> {
    todo!()
}

pub fn update_server(
    db: &mut PgConnection,
    server_id: Uuid,
    host: IpAddr,
    port: u16,
    r#type: Option<String>,
) -> Result<(), Error> {
    todo!()
}

// Spectator

pub fn add_spectators(
    db: &mut PgConnection,
    steamid: Vec<String>,
    match_id: Uuid,
) -> Result<(), Error> {
    todo!()
}

pub fn remove_spectators(
    db: &mut PgConnection,
    steamid: Vec<String>,
    match_id: Uuid,
) -> Result<(), Error> {
    todo!()
}

pub async fn get_spectators(
    db: &mut PgConnection,
    match_id: Uuid,
) -> Result<Option<Vec<String>>, Error> {
    let query: sqlx::Result<Vec<Spectator>> =
        sqlx::query_as!(Spectator, "SELECT * FROM spectators WHERE id IN (SELECT spectator_id FROM match_spectator WHERE match_id = $1)", match_id)
            .fetch_all(db)
            .await;

    match query {
        Ok(mut rows) => {
            let steamids = rows.drain(..).map(|row| row.steamid).collect();
            Ok(Some(steamids))
        }
        Err(err) => match err {
            sqlx::Error::RowNotFound => Ok(None),
            _ => Err(err.into()),
        },
    }
}

// Team

pub async fn get_team(db: &mut PgConnection, team_id: Uuid) -> Result<Option<Team>, Error> {
    let query: sqlx::Result<Team> =
        sqlx::query_as!(Team, "SELECT * FROM teams WHERE id = $1", team_id)
            .fetch_one(db)
            .await;

    match query {
        Ok(team) => Ok(Some(team)),
        Err(err) => match err {
            sqlx::Error::RowNotFound => Ok(None),
            _ => Err(err.into()),
        },
    }
}

pub fn create_team(
    db: &mut PgConnection,
    name: String,
    country: Option<CountryCode>,
    logo: Option<String>,
) -> Result<(), Error> {
    todo!()
}

pub fn update_team(
    db: &mut PgConnection,
    team_id: i32,
    name: String,
    country: Option<CountryCode>,
    logo: Option<String>,
) -> Result<(), Error> {
    todo!()
}

pub fn delete_team(db: &mut PgConnection, team_id: Uuid) -> Result<(), Error> {
    todo!()
}

pub fn get_teams(db: &mut PgConnection) -> Result<Vec<Team>, Error> {
    todo!()
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("An error occurred in the underlying database driver")]
    DatabaseError(#[from] sqlx::Error),
}
