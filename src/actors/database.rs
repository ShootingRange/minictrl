use actix::prelude::*;
use crate::database::models;
use diesel::{PgConnection, RunQueryDsl};
use crate::database::models::{Team, NewTeam};

pub struct DbExecutor {
    pub conn: PgConnection,
}

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

impl Message for NewTeam {
    type Result = Result<models::Team, diesel::result::Error>;
}

impl Handler<NewTeam> for DbExecutor {
    type Result = Result<models::Team, diesel::result::Error>;

    fn handle(&mut self, msg: NewTeam, _: &mut Self::Context) -> Self::Result {
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

    fn handle(&mut self, _msg: ListTeams, _ctx: &mut Self::Context) -> Self::Result {
        use crate::database::schema::teams::dsl::*;

        teams.load::<Team>(&self.conn)
    }
}
