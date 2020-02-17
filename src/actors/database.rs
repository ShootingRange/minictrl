use actix::prelude::*;
use diesel::PgConnection;

pub struct DbExecutor {
    pub conn: PgConnection,
}

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}


pub mod player {
    use actix::Message;
    use std::error::Error;
    use crate::database::models::Player;

    pub struct CreatePlayer {
        // TODO
    }

    impl Message for CreatePlayer {
        type Result = Result<Player, Box<dyn Error>>;
    }

    pub struct FindPlayerById {
        id: i32
    }

    impl Message for FindPlayerById {
        type Result = Result<Player, Box<dyn Error>>;
    }

    pub struct UpdatePlayer {
        // TODO
    }

    impl Message for UpdatePlayer {
        type Result = Result<Player, Box<dyn Error>>;
    }

    pub struct DeletePlayerById {
        id: i32,
    }

    impl Message for DeletePlayerById {
        type Result = Result<(), Box<dyn Error>>;
    }

    pub struct FindPlayersByTeamId {
        team_id: i32,
    }

    impl Message for FindPlayersByTeamId {
        type Result = Result<Vec<Player>, Box<dyn Error>>;
    }
}

pub mod team {
    use actix::{Message, Handler};
    use std::error::Error;
    use crate::database::models::Team;
    use super::DbExecutor;
    use diesel::prelude::*;

    pub struct CreateTeam {
        // TODO
    }

    impl Message for CreateTeam {
        type Result = Result<Team, Box<dyn Error>>;
    }

    pub struct FindTeamById {
        id: i32,
    }

    impl Message for FindTeamById {
        type Result = Result<Team, Box<dyn Error>>;
    }

    impl Handler<FindTeamById> for DbExecutor {
        type Result = Result<Team, Box<dyn Error>>;

        fn handle(&mut self, msg: FindTeamById, ctx: &mut Self::Context) -> Self::Result {
            use crate::database::schema::teams::dsl::*;

            match teams.filter(id.eq(msg.id)).first::<Team>(&self.conn) {
                Ok(t) => Ok(t),
                Err(err) => Err(Box::new(err)),
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
        id: i32,
    }

    impl Message for DeleteTeamById {
        type Result = Result<(), Box<dyn Error>>;
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
