use actix::prelude::*;
use diesel::PgConnection;
use std::fmt::Display;
use serde::export::Formatter;
use serde::export::fmt::Error;

pub struct DbExecutor {
    pub conn: PgConnection,
}

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

#[derive(Debug)]
pub enum DbActorError {
    DatabaseError(diesel::result::Error)
}

impl Display for DbActorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            DbActorError::DatabaseError(err) => err.fmt(f),
        }
    }
}

pub mod player {
    use actix::{Message, Handler};
    use std::error::Error;
    use crate::database::models::Player;
    use crate::actors::database::{DbExecutor, DbActorError};
    use diesel::prelude::*;

    pub struct CreatePlayer {
        // TODO
    }

    impl Message for CreatePlayer {
        type Result = Result<Player, Box<dyn Error>>;
    }

    pub struct FindPlayerById {
        pub id: i32
    }

    impl Message for FindPlayerById {
        type Result = Result<Player, DbActorError>;
    }

    impl Handler<FindPlayerById> for DbExecutor {
        type Result = Result<Player, DbActorError>;

        fn handle(&mut self, msg: FindPlayerById, _ctx: &mut Self::Context) -> Self::Result {
            use crate::database::schema::players::dsl::*;

            match players.filter(id.eq(msg.id)).first::<Player>(&self.conn) {
                Ok(t) => Ok(t),
                Err(err) => Err(DbActorError::DatabaseError(err)),
            }
        }
    }

    pub struct UpdatePlayer {
        // TODO
    }

    impl Message for UpdatePlayer {
        type Result = Result<Player, Box<dyn Error>>;
    }

    pub struct DeletePlayerById {
        pub id: i32,
    }

    impl Message for DeletePlayerById {
        type Result = Result<(), Box<dyn Error>>;
    }

    pub struct FindPlayersByTeamId {
        pub team_id: i32,
    }

    impl Message for FindPlayersByTeamId {
        type Result = Result<Vec<Player>, DbActorError>;
    }

    impl Handler<FindPlayersByTeamId> for DbExecutor {
        type Result = Result<Vec<Player>, DbActorError>;

        fn handle(&mut self, msg: FindPlayersByTeamId, _ctx: &mut Self::Context) -> Self::Result {
            use crate::database::schema::players::dsl::*;

            match players.filter(team_id.eq(msg.team_id)).load::<Player>(&self.conn) {
                Ok(ps) => Ok(ps),
                Err(err) => Err(DbActorError::DatabaseError(err)),
            }
        }
    }
}

pub mod team {
    use actix::{Message, Handler};
    use std::error::Error;
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
        type Result = Result<Team, Box<dyn Error>>;
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
        type Result = Result<Vec<Team>, Box<dyn Error>>;
    }

    impl Handler<GetTeams> for DbExecutor {
        type Result = Result<Vec<Team>, Box<dyn Error>>;

        fn handle(&mut self, _msg: GetTeams, _ctx: &mut Self::Context) -> Self::Result {
            use crate::database::schema::teams::dsl::*;

            match teams.load::<Team>(&self.conn) {
                Ok(t) => Ok(t),
                Err(err) => Err(Box::new(err)),
            }
        }
    }
}

/*impl Message for NewTeam {
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
}*/
