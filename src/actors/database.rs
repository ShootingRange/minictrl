use actix::prelude::*;
use crate::database::models;
use diesel::{PgConnection, RunQueryDsl};
use crate::database::models::{CountryCode, Team};
use crate::database::schema::*;

pub struct DbExecutor {
    pub conn: PgConnection,
}

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

#[derive(Insertable)]
#[table_name = "teams"]
pub struct CreateTeam {
    pub name: String,
    pub country: Option<CountryCode>,
    pub logo: Option<String>,
}

impl Message for CreateTeam {
    type Result = Result<models::Team, diesel::result::Error>;
}

impl Handler<CreateTeam> for DbExecutor {
    type Result = Result<models::Team, diesel::result::Error>;

    fn handle(&mut self, msg: CreateTeam, _: &mut Self::Context) -> Self::Result {
        use crate::database::schema::teams::dsl::*;

        let team = diesel::insert_into(teams)
            .values(&msg)
            .get_result(&self.conn)?;

        Ok(team)
    }
}

pub struct ListTeams {}

impl Message for ListTeams {
    type Result = Result<Vec<models::Team>, diesel::result::Error>;
}

impl Handler<ListTeams> for DbExecutor {
    type Result = Result<Vec<models::Team>, diesel::result::Error>;

    fn handle(&mut self, msg: ListTeams, ctx: &mut Self::Context) -> Self::Result {
        use crate::database::schema::teams::dsl::*;

        teams.load::<Team>(&self.conn)
    }
}
