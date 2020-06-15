use crate::actors::database::{DbActorError, DbExecutor};
use crate::common::SideType;
use crate::database::models::{Match, MatchSpectator, NewMatch, NewMatchSpectator};
use actix::{Handler, Message};
use diesel::prelude::*;

pub struct CreateMatch {
    pub server_id: i32,
    pub team1_id: i32,
    pub team2_id: i32,
    pub team1_score: Option<i32>,
    pub team2_score: Option<i32>,
    pub num_maps: i32,
    pub skip_veto: bool,
    pub veto_first: SideType,
    pub players_per_team: i32,
    pub min_player_to_ready: i32,
}

impl Message for CreateMatch {
    type Result = Result<Match, DbActorError>;
}

impl Handler<CreateMatch> for DbExecutor {
    type Result = Result<Match, DbActorError>;

    fn handle(&mut self, msg: CreateMatch, _ctx: &mut Self::Context) -> Self::Result {
        use crate::database::schema::matches::dsl::*;

        let r#match = NewMatch {
            server_id: msg.server_id,
            team1_id: msg.team1_id,
            team2_id: msg.team2_id,
            team1_score: msg.team1_score,
            team2_score: msg.team2_score,
            num_maps: msg.num_maps,
            skip_veto: msg.skip_veto,
            veto_first: msg.veto_first,
            players_per_team: msg.players_per_team,
            min_player_to_ready: msg.min_player_to_ready,
        };

        diesel::insert_into(matches)
            .values(&r#match)
            .get_result::<Match>(&self.conn)
            .map_err(DbActorError::DatabaseError)
    }
}

pub struct FindMatchById {
    pub id: i32,
}

impl Message for FindMatchById {
    type Result = Result<Match, DbActorError>;
}

impl Handler<FindMatchById> for DbExecutor {
    type Result = Result<Match, DbActorError>;

    fn handle(&mut self, msg: FindMatchById, _ctx: &mut Self::Context) -> Self::Result {
        use crate::database::schema::matches::dsl::*;

        match matches.filter(id.eq(msg.id)).first::<Match>(&self.conn) {
            Ok(t) => Ok(t),
            Err(err) => Err(DbActorError::DatabaseError(err)),
        }
    }
}

pub struct UpdateMatch {
    // TODO
}

impl Message for UpdateMatch {
    type Result = Result<Match, DbActorError>;
}

pub struct DeleteMatchById {
    pub id: i32,
}

impl Message for DeleteMatchById {
    type Result = Result<bool, DbActorError>;
}

impl Handler<DeleteMatchById> for DbExecutor {
    type Result = Result<bool, DbActorError>;

    fn handle(&mut self, msg: DeleteMatchById, _ctx: &mut Self::Context) -> Self::Result {
        use crate::database::schema::matches::dsl::*;

        diesel::delete(matches.filter(id.eq(msg.id)))
            .execute(&self.conn)
            .map_err(DbActorError::DatabaseError)
            .map(|size| size > 0)
    }
}

pub struct AddSpectatorToMatch {
    spectator_id: i32,
    match_id: i32,
}

impl Message for AddSpectatorToMatch {
    type Result = Result<MatchSpectator, DbActorError>;
}

impl Handler<AddSpectatorToMatch> for DbExecutor {
    type Result = Result<MatchSpectator, DbActorError>;

    fn handle(&mut self, msg: AddSpectatorToMatch, _ctx: &mut Self::Context) -> Self::Result {
        use crate::database::schema::match_spectator::dsl::*;

        let relation = NewMatchSpectator {
            match_id: msg.match_id,
            spectator_id: msg.spectator_id,
        };

        diesel::insert_into(match_spectator)
            .values(relation)
            .get_result::<MatchSpectator>(&self.conn)
            .map_err(DbActorError::DatabaseError)
    }
}

pub struct RemoveSpectatorFromMatch {
    spectator_id: i32,
    match_id: i32,
}

impl Message for RemoveSpectatorFromMatch {
    type Result = Result<bool, DbActorError>;
}

impl Handler<RemoveSpectatorFromMatch> for DbExecutor {
    type Result = Result<bool, DbActorError>;

    fn handle(&mut self, msg: RemoveSpectatorFromMatch, _ctx: &mut Self::Context) -> Self::Result {
        use crate::database::schema::match_spectator::dsl::*;

        diesel::delete(
            match_spectator
                .filter(spectator_id.eq(msg.spectator_id))
                .filter(match_id.eq(msg.match_id)),
        )
        .execute(&self.conn)
        .map_err(DbActorError::DatabaseError)
        .map(|size| size > 0)
    }
}

/*pub struct AssignMatchToServer {
    server_id: i32,
    match_id: i32,
}

impl Message for AssignMatchToServer {
    type Result = Result<(), DbActorError>;
}

impl Handler<AssignMatchToServer> for DbExecutor {
    type Result = Result<(), DbActorError>;

    fn handle(&mut self, msg: AssignMatchToServer, _ctx: &mut Self::Context) -> Self::Result {
        // TODO
        unimplemented!()
    }
}*/
