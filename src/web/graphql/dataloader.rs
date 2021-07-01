use std::collections::HashMap;

use async_graphql::dataloader::Loader;
use async_graphql::FieldError;
use itertools::Itertools;
use sqlx::types::Uuid;
use sqlx::{Pool, Postgres};
use IntoIterator;

use crate::database::models::{Match, Player, Team};

/// Builds a list of UUIDs for use in a SQL query
fn uuid_list(keys: &[Uuid]) -> String {
    keys.iter()
        .map(|key| format!("'{}'", key.to_string()))
        .enumerate()
        .fold(String::new(), |mut acc, (i, key)| {
            if i == 0 {
                key.to_string()
            } else {
                acc.push_str(", ");
                acc.push_str(key.as_str());
                acc
            }
        })
}

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
        let key_list = uuid_list(keys);
        let query = format!("SELECT * FROM teams WHERE id IN ({})", key_list);

        Ok(sqlx::query_as::<_, Team>(query.as_str())
            .fetch_all(&self.0)
            .await?
            .drain(..)
            .map(|row| (row.id, row))
            .collect())
    }
}

pub struct PlayerLoader(Pool<Postgres>);

impl PlayerLoader {
    pub(in crate::web::graphql) fn new(postgres_pool: Pool<Postgres>) -> Self {
        Self(postgres_pool)
    }
}

#[async_trait]
impl Loader<Uuid> for PlayerLoader {
    type Value = Player;
    type Error = FieldError;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let key_list = uuid_list(keys);
        let query = format!("SELECT * FROM teams WHERE id IN ({})", key_list);

        Ok(sqlx::query_as::<_, Player>(query.as_str())
            .fetch_all(&self.0)
            .await?
            .drain(..)
            .map(|row| (row.id, row))
            .collect())
    }
}

pub struct PlayerTeamLoader(Pool<Postgres>);

impl PlayerTeamLoader {
    pub(in crate::web::graphql) fn new(postgres_pool: Pool<Postgres>) -> Self {
        Self(postgres_pool)
    }
}

#[async_trait]
impl Loader<Uuid> for PlayerTeamLoader {
    type Value = Vec<Player>;
    type Error = FieldError;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let key_list = uuid_list(keys);
        let query = format!("SELECT * FROM players WHERE team_id IN ({})", key_list);

        let mut players = sqlx::query_as::<_, Player>(query.as_str())
            .fetch_all(&self.0)
            .await?;
        players.sort_by_key(|player| player.team_id);
        let team_players: HashMap<Uuid, Self::Value> = players
            .iter()
            .group_by(|player| player.team_id)
            .into_iter()
            .map(|(team_id, group)| (team_id, group.cloned().collect::<Vec<Player>>()))
            .collect();

        Ok(team_players)
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
        let key_list = uuid_list(keys);
        let query = format!("SELECT * FROM teams WHERE id IN ({})", key_list);

        Ok(sqlx::query_as::<_, Match>(query.as_str())
            .fetch_all(&self.0)
            .await?
            .drain(..)
            .map(|row| (row.id, row))
            .collect())
    }
}

// TODO implement loaders for all structs in crate::database::models
