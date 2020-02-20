use actix::prelude::*;
use diesel::PgConnection;
use std::fmt::Display;
use serde::export::Formatter;
use serde::export::fmt::Error;

pub mod player;
pub mod team;

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
