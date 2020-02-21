use crate::actors::database::{DbActorError, DbExecutor};
use crate::database::models::{NewSpectator, Spectator};
use actix::{Handler, Message};
use diesel::prelude::*;

pub struct CreateSpectator {
    pub steamid: String,
}

impl Message for CreateSpectator {
    type Result = Result<Spectator, DbActorError>;
}

impl Handler<CreateSpectator> for DbExecutor {
    type Result = Result<Spectator, DbActorError>;

    fn handle(&mut self, msg: CreateSpectator, _ctx: &mut Self::Context) -> Self::Result {
        use crate::database::schema::spectators::dsl::*;

        let spectator = NewSpectator {
            steamid: msg.steamid,
        };

        diesel::insert_into(spectators)
            .values(&spectator)
            .get_result::<Spectator>(&self.conn)
            .map_err(|err| DbActorError::DatabaseError(err))
    }
}

pub struct FindSpectatorById {
    pub id: i32,
}

impl Message for FindSpectatorById {
    type Result = Result<Spectator, DbActorError>;
}

impl Handler<FindSpectatorById> for DbExecutor {
    type Result = Result<Spectator, DbActorError>;

    fn handle(&mut self, msg: FindSpectatorById, _ctx: &mut Self::Context) -> Self::Result {
        use crate::database::schema::spectators::dsl::*;

        match spectators
            .filter(id.eq(msg.id))
            .first::<Spectator>(&self.conn)
        {
            Ok(t) => Ok(t),
            Err(err) => Err(DbActorError::DatabaseError(err)),
        }
    }
}

pub struct UpdateSpectator {
    // TODO
}

impl Message for UpdateSpectator {
    type Result = Result<Spectator, DbActorError>;
}

pub struct DeleteSpectatorById {
    pub id: i32,
}

impl Message for DeleteSpectatorById {
    type Result = Result<bool, DbActorError>;
}

impl Handler<DeleteSpectatorById> for DbExecutor {
    type Result = Result<bool, DbActorError>;

    fn handle(&mut self, msg: DeleteSpectatorById, _ctx: &mut Self::Context) -> Self::Result {
        use crate::database::schema::spectators::dsl::*;

        diesel::delete(spectators.filter(id.eq(msg.id)))
            .execute(&self.conn)
            .map_err(|err| DbActorError::DatabaseError(err))
            .map(|size| size > 0)
    }
}
