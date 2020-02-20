use crate::actors::database::{DbActorError, DbExecutor};
use crate::database::models::{NewPlayer, Player, Server};
use actix::{Handler, Message};
use diesel::prelude::*;

pub struct CreatePlayer {
    pub team_id: i32,
    pub name: String,
    pub tag: Option<String>,
    pub steamid: Option<String>,
}

impl Message for CreatePlayer {
    type Result = Result<Player, DbActorError>;
}

impl Handler<CreatePlayer> for DbExecutor {
    type Result = Result<Player, DbActorError>;

    fn handle(&mut self, msg: CreatePlayer, _ctx: &mut Self::Context) -> Self::Result {
        use crate::database::schema::players::dsl::*;

        let player = NewPlayer {
            team_id: msg.team_id,
            name: msg.name,
            tag: msg.tag,
            steamid: msg.steamid,
        };

        diesel::insert_into(players)
            .values(&player)
            .get_result::<Player>(&self.conn)
            .map_err(|err| DbActorError::DatabaseError(err))
    }
}

pub struct FindPlayerById {
    pub id: i32,
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
    type Result = Result<Player, DbActorError>;
}

pub struct DeletePlayerById {
    pub id: i32,
}

impl Message for DeletePlayerById {
    type Result = Result<bool, DbActorError>;
}

impl Handler<DeletePlayerById> for DbExecutor {
    type Result = Result<bool, DbActorError>;

    fn handle(&mut self, msg: DeletePlayerById, _ctx: &mut Self::Context) -> Self::Result {
        use crate::database::schema::players::dsl::*;

        diesel::delete(players.filter(id.eq(msg.id)))
            .execute(&self.conn)
            .map_err(|err| DbActorError::DatabaseError(err))
            .map(|size| size > 0)
    }
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

        match players
            .filter(team_id.eq(msg.team_id))
            .load::<Player>(&self.conn)
        {
            Ok(ps) => Ok(ps),
            Err(err) => Err(DbActorError::DatabaseError(err)),
        }
    }
}

pub struct FindServerById {
    pub id: i32,
}

impl Message for FindServerById {
    type Result = Result<Server, DbActorError>;
}

impl Handler<FindServerById> for DbExecutor {
    type Result = Result<Server, DbActorError>;

    fn handle(&mut self, msg: FindServerById, _ctx: &mut Self::Context) -> Self::Result {
        use crate::database::schema::servers::dsl::*;

        match servers.filter(id.eq(msg.id)).first::<Server>(&self.conn) {
            Ok(t) => Ok(t),
            Err(err) => Err(DbActorError::DatabaseError(err)),
        }
    }
}
