use std::collections::HashMap;

use async_graphql::dataloader::Loader;
use async_graphql::FieldError;
use sqlx::types::Uuid;
use sqlx::{Pool, Postgres};

use crate::database::models::{Match, Team};

pub struct TeamLoader(Pool<Postgres>);

impl TeamLoader {
    pub(in crate::web::graphql) fn new(postgres_pool: Pool<Postgres>) -> Self {
        Self(postgres_pool)
    }
}

#[async_trait]
impl Loader<Uuid> for TeamLoader {
    type Value = Team;
    type Error = FieldError;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let mut map = HashMap::with_capacity(keys.len());

        for key in keys {
            /*map.insert(
                *key,
                todo!(),
            );*/
            todo!();
        }

        Ok(map)
    }
}

pub struct MatchLoader(Pool<Postgres>);

impl MatchLoader {
    pub(in crate::web::graphql) fn new(postgres_pool: Pool<Postgres>) -> Self {
        Self(postgres_pool)
    }
}

#[async_trait]
impl Loader<Uuid> for MatchLoader {
    type Value = Match;
    type Error = FieldError;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let mut map = HashMap::with_capacity(keys.len());

        for key in keys {
            /*map.insert(
                *key,
                todo!(),
            );*/
            todo!();
        }

        Ok(map)
    }
}

// TODO implement loaders for all structs in crate::database::models
