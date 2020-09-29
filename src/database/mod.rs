pub mod models;
pub mod schema;

use crate::common::SideType;
use crate::database::models::{Match, Player, Server, Team, CountryCode, MapList};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use serde::export::Formatter;
use std::net::IpAddr;

pub struct Database {
    conn_pool: Pool<ConnectionManager<PgConnection>>,
}

impl Database {
    pub fn new(conn_pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Database { conn_pool }
    }

    #[deprecated]
    pub fn find_team_by_id(&self, team_id: i32) -> Result<Option<Team>, Error> {
        use crate::database::schema::teams::dsl::*;

        let conn = self.conn_pool.get().map_err(Error::Pool)?;

        match teams.filter(id.eq(&team_id)).first::<Team>(&conn) {
            Ok(t) => Ok(Some(t)),
            Err(err) => Err(Error::DB(err)), // TODO if database error not found return Option::None
        }
    }

    #[deprecated]
    pub fn find_match_by_id(&self, match_id: i32) -> Result<Option<Match>, Error> {
        use crate::database::schema::matches::dsl::*;

        let conn = self.conn_pool.get().map_err(Error::Pool)?;

        match matches.filter(id.eq(&match_id)).first::<Match>(&conn) {
            Ok(t) => Ok(Some(t)),
            Err(err) => Err(Error::DB(err)), // TODO if database eror not found return Option::None
        }
    }

    // Match

    pub fn set_map_list_for_match(&self, match_id: i32, maps: Vec<String>) {
        unimplemented!()
    }

    pub fn get_match_map_list(&self, match_id: i32) -> Result<Option<Vec<String>>, Error> {
        use crate::database::schema::maplist::dsl;

        let conn = self.conn_pool.get().map_err(Error::Pool)?;

        // Check if the match exists, so we can return None
        match self.get_match(match_id) {
            Ok(_) => {}
            Err(err) => match err {
                Error::DB(err) => match err {
                    diesel::result::Error::NotFound => return Ok(None),
                    _ => return Err(Error::DB(err)),
                },
                _ => return Err(err),
            },
        }

        // Get maplist records for the given match
        match dsl::maplist
            .filter(dsl::match_id.eq(&match_id))
            .load::<MapList>(&conn)
        {
            Ok(mut maps) => {
                // Sort the map by the order written in the database
                maps.sort_by_key(|map| map.order);

                // Strip record down to the map name
                let map_names: Vec<String> = maps.iter().map(|map| map.map.to_string()).collect();

                Ok(Some(map_names))
            }
            Err(err) => Err(Error::DB(err)),
        }
    }

    pub fn create_match(
        &self,
        server_id: i32,
        team1_id: i32,
        team2_id: i32,
        team1_score: Option<i32>,
        team2_score: Option<i32>,
        num_maps: i32,
        skip_veto: bool,
        veto_first: SideType,
        players_per_team: i32,
        min_player_to_ready: i32,
    ) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn get_match(&self, match_id: i32) -> Result<Option<Match>, Error> {
        unimplemented!()
    }

    pub fn update_match(
        &self,
        match_id: i32,
        server_id: i32,
        team1_id: i32,
        team2_id: i32,
        team1_score: Option<i32>,
        team2_score: Option<i32>,
        num_maps: i32,
        skip_veto: bool,
        veto_first: SideType,
        players_per_team: i32,
        min_player_to_ready: i32,
    ) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn delete_match(&self, match_id: i32) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn add_spectator_to_match(&self, spectator_id: i32, match_id: i32) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn remove_spectator_from_match(
        &self,
        spectator_id: i32,
        match_id: i32,
    ) -> Result<(), Error> {
        unimplemented!()
    }

    // Player

    // TODO index the player by steamid
    pub fn add_player_to_team(
        &self,
        team_id: i32,
        name: String,
        tag: Option<String>,
        steamid: Option<String>,
    ) -> Result<i32, Error> {
        unimplemented!()
    }

    pub fn remove_player_from_team(&self, player_id: i32) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn get_player(&self, player_id: i32) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn update_player(&self, player_id: i32) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn get_team_players(&self, team_id: i32) -> Result<Option<Vec<Player>>, Error> {
        use crate::database::schema::players::dsl;

        let conn = self.conn_pool.get().map_err(Error::Pool)?;

        match dsl::players.filter(dsl::team_id.eq(team_id)).load(&conn) {
            Ok(t) => Ok(Some(t)),
            Err(err) => match err {
                Error::NotFound => Ok(None),
                _ => Err(Error::DB(err)),
            },
        }
    }

    // Server

    pub fn get_server(&self, server_id: i32) -> Result<Option<Server>, Error> {
        use crate::database::schema::servers::dsl;

        let conn = self.conn_pool.get().map_err(Error::Pool)?;

        match dsl::servers.filter(dsl::id.eq(server_id)).load(&conn) {
            Ok(t) => Ok(Some(t)),
            Err(err) => match err {
                Error::NotFound => Ok(None),
                _ => Err(Error::DB(err)),
            },
        }
    }

    pub fn add_server(&self, host: IpAddr, port: u16, r#type: Option<String>) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn remove_server(&self, server_id: i32) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn server_info(&self, server_id: i32) -> Result<Option<Server>, Error> {
        unimplemented!()
    }

    pub fn update_server(
        &self,
        server_id: i32,
        host: IpAddr,
        port: u16,
        r#type: Option<String>,
    ) -> Result<(), Error> {
        unimplemented!()
    }

    // Spectator

    pub fn add_spectators(&self, steamid: Vec<String>, match_id: i32) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn remove_spectators(&self, steamid: Vec<String>, match_id: i32) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn get_spectators(&self, match_id: i32) -> Result<Option<Vec<String>>, Error> {
        unimplemented!()
    }

    // Team

    pub fn get_team(&self, team_id: i32) -> Result<Option<Team>, Error> {
        use crate::database::schema::teams::dsl;

        let conn = self.conn_pool.get().map_err(Error::Pool)?;

        match dsl::teams.filter(dsl::id.eq(team_id)).load(&conn) {
            Ok(t) => Ok(Some(t)),
            Err(err) => match err {
                Error::NotFound => Ok(None),
                _ => Err(Error::DB(err)),
            },
        }
    }

    pub fn create_team(&self, name: String, country: Option<CountryCode>, logo: Option<String>) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn update_team(&self, team_id: i32, name: String, country: Option<CountryCode>, logo: Option<String>) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn delete_team(&self, team_id: i32) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn get_teams(&self) -> Result<Vec<Team>, Error> {
        use crate::database::schema::servers::dsl;

        let conn = self.conn_pool.get().map_err(Error::Pool)?;

        match dsl::servers.load(&conn) {
            Ok(teams) => Ok(teams),
            Err(err) => Err(Error::DB(err)),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    DB(diesel::result::Error),
    Pool(diesel::r2d2::PoolError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DB(err) => write!(f, "Database Error: {}", err),
            Error::Pool(err) => write!(f, "Pool Error: {}", err),
        }
    }
}
