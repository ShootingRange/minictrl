use crate::actors::database::{DbActorError, DbExecutor};
use crate::database::models::{MapList, NewMapList};
use actix::{Handler, Message};
use diesel::prelude::*;

pub struct SetMapListForMatch {
    pub match_id: i32,
    pub maps: Vec<String>,
}

impl Message for SetMapListForMatch {
    type Result = Result<(), DbActorError>;
}

impl Handler<SetMapListForMatch> for DbExecutor {
    type Result = Result<(), DbActorError>;

    fn handle(&mut self, msg: SetMapListForMatch, _ctx: &mut Self::Context) -> Self::Result {
        use crate::database::schema::maplist::dsl::*;

        self.conn
            .transaction(|| {
                diesel::delete(maplist.filter(match_id.eq(msg.match_id))).execute(&self.conn)?;

                let maps: Vec<NewMapList> = msg
                    .maps
                    .iter()
                    .enumerate()
                    .map(|(i, name)| NewMapList {
                        match_id: msg.match_id,
                        order: i as i32,
                        map: name.clone(),
                    })
                    .collect();

                diesel::insert_into(maplist)
                    .values(&maps)
                    .execute(&self.conn)?;

                Ok(())
            })
            .map_err(DbActorError::DatabaseError)?;

        Ok(())
    }
}

pub struct FindMapsByMatchId {
    pub match_id: i32,
}

impl Message for FindMapsByMatchId {
    type Result = Result<Vec<String>, DbActorError>;
}

impl Handler<FindMapsByMatchId> for DbExecutor {
    type Result = Result<Vec<String>, DbActorError>;

    fn handle(&mut self, msg: FindMapsByMatchId, _ctx: &mut Self::Context) -> Self::Result {
        use crate::database::schema::maplist::dsl::*;

        let mut entries: Vec<MapList> = maplist
            .filter(id.eq(msg.match_id))
            .load::<MapList>(&self.conn)
            .map_err(DbActorError::DatabaseError)?;

        // Sort by order
        entries.sort_by(|a, b| a.order.cmp(&b.order));

        // Extract map name
        let maps: Vec<String> = entries.iter().map(|entry| entry.map.clone()).collect();

        Ok(maps)
    }
}
