pub mod models;
pub mod schema;

use crate::database::models::{Match, Player, Team};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

pub struct Database {
    conn_pool: Pool<ConnectionManager<PgConnection>>,
}

impl Database {
    pub fn new(conn_pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Database { conn_pool }
    }

    pub fn find_team_by_id(&self, team_id: i32) -> Result<Team, Error> {
        use crate::database::schema::teams::dsl::*;

        let conn = self.conn_pool.get().map_err(|err| Error::Pool(err))?;

        match teams.filter(id.eq(&team_id)).first::<Team>(&conn) {
            Ok(t) => Ok(t),
            Err(err) => Err(Error::DB(err)),
        }
    }

    pub fn find_match_by_id(&self, match_id: i32) -> Result<Match, Error> {
        use crate::database::schema::matches::dsl::*;

        let conn = self.conn_pool.get().map_err(|err| Error::Pool(err))?;

        match matches.filter(id.eq(&match_id)).first::<Match>(&conn) {
            Ok(t) => Ok(t),
            Err(err) => Err(Error::DB(err)),
        }
    }

    pub fn get_team_players(&self, team_id: i32) -> Result<Vec<Player>, Error> {
        use crate::database::schema::players::dsl;

        let conn = self.conn_pool.get().map_err(|err| Error::Pool(err))?;

        match dsl::players.filter(dsl::team_id.eq(team_id)).load(&conn) {
            Ok(t) => Ok(t),
            Err(err) => Err(Error::DB(err)),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    DB(diesel::result::Error),
    Pool(diesel::r2d2::PoolError),
}
