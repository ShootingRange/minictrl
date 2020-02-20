use actix::{Message, Handler};
use crate::database::models::{Team, CountryCode, NewTeam};
use super::DbExecutor;
use diesel::prelude::*;
use crate::actors::database::DbActorError;

pub struct CreateTeam {
    pub name: String,
    pub country: Option<CountryCode>,
    pub logo: Option<String>,
}

impl Message for CreateTeam {
    type Result = Result<Team, DbActorError>;
}

impl Handler<CreateTeam> for DbExecutor {
    type Result = Result<Team, DbActorError>;

    fn handle(&mut self, msg: CreateTeam, _ctx: &mut Self::Context) -> Self::Result {
        use crate::database::schema::teams::dsl::*;

        let team = NewTeam {
            name: msg.name,
            country: msg.country,
            logo: msg.logo,
        };

        diesel::insert_into(teams)
            .values(&team)
            .get_result::<Team>(&self.conn)
            .map_err(|err| DbActorError::DatabaseError(err))
    }
}

pub struct FindTeamById {
    pub id: i32,
}

impl Message for FindTeamById {
    type Result = Result<Team, DbActorError>;
}

impl Handler<FindTeamById> for DbExecutor {
    type Result = Result<Team, DbActorError>;

    fn handle(&mut self, msg: FindTeamById, _ctx: &mut Self::Context) -> Self::Result {
        use crate::database::schema::teams::dsl::*;

        match teams.filter(id.eq(msg.id)).first::<Team>(&self.conn) {
            Ok(t) => Ok(t),
            Err(err) => Err(DbActorError::DatabaseError(err)),
        }
    }
}

pub struct UpdateTeam {
    // TODO
}

impl Message for UpdateTeam {
    type Result = Result<Team, DbActorError>;
}

pub struct DeleteTeamById {
    pub id: i32,
}

impl Message for DeleteTeamById {
    type Result = Result<bool, DbActorError>;
}

impl Handler<DeleteTeamById> for DbExecutor {
    type Result = Result<bool, DbActorError>;

    fn handle(&mut self, msg: DeleteTeamById, _ctx: &mut Self::Context) -> Self::Result {
        use crate::database::schema::teams::dsl::*;

        diesel::delete(teams.filter(id.eq(msg.id)))
            .execute(&self.conn)
            .map_err(|err| DbActorError::DatabaseError(err))
            .map(|size| size > 0)
    }
}

pub struct GetTeams {}

impl Message for GetTeams {
    type Result = Result<Vec<Team>, DbActorError>;
}

impl Handler<GetTeams> for DbExecutor {
    type Result = Result<Vec<Team>, DbActorError>;

    fn handle(&mut self, _msg: GetTeams, _ctx: &mut Self::Context) -> Self::Result {
        use crate::database::schema::teams::dsl::*;

        match teams.load::<Team>(&self.conn) {
            Ok(t) => Ok(t),
            Err(err) => Err(DbActorError::DatabaseError(err)),
        }
    }
}
