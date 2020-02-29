use actix::prelude::*;
use diesel::PgConnection;
use serde::export::fmt::Error;
use serde::export::Formatter;
use std::fmt::Display;

pub mod maps;
pub mod r#match;
pub mod player;
pub mod server;
pub mod spectator;
pub mod team;

pub struct DbExecutor {
    pub conn: PgConnection,
}

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

#[derive(Debug)]
pub enum DbActorError {
    DatabaseError(diesel::result::Error),
}

impl Display for DbActorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            DbActorError::DatabaseError(err) => err.fmt(f),
        }
    }
}
