use crate::actors::database::{DbActorError, DbExecutor};
use crate::database::models::{MatchSpectator, NewSpectator, Spectator};
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

pub struct FindSpectatorsByMatch {
    pub id: i32,
}

impl Message for FindSpectatorsByMatch {
    type Result = Result<Vec<Spectator>, DbActorError>;
}

impl Handler<FindSpectatorsByMatch> for DbExecutor {
    type Result = Result<Vec<Spectator>, DbActorError>;

    fn handle(&mut self, msg: FindSpectatorsByMatch, _ctx: &mut Self::Context) -> Self::Result {
        use crate::database::schema::{match_spectator, spectators};

        let match_spectators: Vec<MatchSpectator> = match_spectator::dsl::match_spectator
            .filter(match_spectator::dsl::match_id.eq(msg.id))
            .load::<MatchSpectator>(&self.conn)
            .map_err(|err| DbActorError::DatabaseError(err))?;

        match_spectators
            .iter()
            .map(|match_spectator| {
                spectators::dsl::spectators
                    .filter(spectators::dsl::id.eq(match_spectator.spectator_id))
                    .first::<Spectator>(&self.conn)
                    .map_err(|err| DbActorError::DatabaseError(err))
            })
            .collect()
    }
}
