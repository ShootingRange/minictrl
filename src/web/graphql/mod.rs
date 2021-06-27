use async_graphql::dataloader::DataLoader;
use async_graphql::extensions::Tracing;
use async_graphql::{Context, EmptySubscription, Schema};
use sqlx::types::Uuid;
use sqlx::{Pool, Postgres};

use dataloader::*;

use crate::common::SideType;
use crate::web::graphql::types::*;

mod dataloader;
mod types;

pub(crate) struct QueryRoot;

#[async_graphql::Object]
impl QueryRoot {
    async fn team(&self, ctx: &Context<'_>, id: Uuid) -> async_graphql::Result<Option<Team>> {
        /*Ok(ctx
        .data_unchecked::<DataLoader<BookLoader>>()
        .load_one(id)
        .await?)*/
        todo!()
    }

    async fn player(&self, ctx: &Context<'_>, id: Uuid) -> async_graphql::Result<Option<Player>> {
        todo!()
    }

    async fn server(&self, ctx: &Context<'_>, id: Uuid) -> async_graphql::Result<Option<Server>> {
        todo!()
    }

    async fn spectator(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
    ) -> async_graphql::Result<Option<Spectator>> {
        todo!()
    }

    async fn r#match(&self, ctx: &Context<'_>, id: Uuid) -> async_graphql::Result<Option<r#Match>> {
        todo!()
    }
}

pub(crate) struct Mutation;

#[async_graphql::Object]
impl Mutation {
    async fn create_team(
        &self,
        name: String,
        country: Option<String>,
        logo: Option<String>,
    ) -> async_graphql::Result<Team> {
        todo!()
    }

    async fn update_team(
        &self,
        id: Uuid,
        name: String,
        country: Option<String>,
        logo: Option<String>,
    ) -> async_graphql::Result<Team> {
        todo!()
    }

    async fn delete_teams(&self, id: Vec<Uuid>) -> async_graphql::Result<Team> {
        todo!()
    }

    async fn add_players(
        &self,
        team: Uuid,
        players: Vec<Uuid>,
    ) -> async_graphql::Result<Vec<Player>> {
        todo!()
    }

    async fn update_player(
        &self,
        team: Uuid,
        steamid: String,
        name: String,
        tag: Option<String>,
    ) -> async_graphql::Result<Player> {
        todo!()
    }

    async fn remove_players(
        &self,
        team: Uuid,
        players: Vec<Uuid>,
    ) -> async_graphql::Result<Vec<Player>> {
        todo!()
    }

    async fn create_server(
        &self,
        host: String,
        port: i32,
        r#type: Option<String>,
        rcon_password: String,
    ) -> async_graphql::Result<Server> {
        todo!()
    }

    async fn update_server(
        &self,
        id: Uuid,
        host: String,
        port: i32,
        r#type: Option<String>,
        rcon_password: String,
    ) -> async_graphql::Result<Server> {
        todo!()
    }

    async fn delete_servers(&self, id: Vec<Uuid>) -> async_graphql::Result<Server> {
        todo!()
    }

    async fn create_match(
        &self,
        server: Option<Uuid>,
        team1: Uuid,
        team2: Uuid,
        num_maps: i32,
        skip_veto: bool,
        veto_first: SideType,
        players_per_team: i32,
        min_player_to_ready: i32,
        maps: Vec<String>,
        spectators: Vec<Uuid>,
    ) -> async_graphql::Result<Server> {
        todo!()
    }

    // TODO update functions for match

    async fn set_map_list_for_match(
        &self,
        r#match: Uuid,
        maps: Vec<String>,
    ) -> async_graphql::Result<bool> {
        todo!()
    }

    async fn delete_matches(&self, id: Vec<Uuid>) -> async_graphql::Result<Server> {
        todo!()
    }

    async fn create_spectator(
        &self,
        steamid: String,
        name: String,
    ) -> async_graphql::Result<Spectator> {
        todo!()
    }

    async fn delete_spectators(&self, steamid: Vec<String>) -> async_graphql::Result<Spectator> {
        todo!()
    }

    /// Add a spectator to a match
    ///
    /// @return: true if the spectator was not already attached to the match
    async fn attach_spectators(
        &self,
        r#match: Uuid,
        spectators: Vec<Uuid>,
    ) -> async_graphql::Result<Vec<bool>> {
        todo!()
    }

    /// Removes a spectator from a match
    ///
    /// @return: true if the spectator was not already detached from the match
    async fn detach_spectators(
        &self,
        r#match: Uuid,
        spectators: Vec<Uuid>,
    ) -> async_graphql::Result<Vec<bool>> {
        todo!()
    }
}

pub(crate) fn init_schema(
    db_pool: Pool<Postgres>,
) -> Schema<QueryRoot, Mutation, EmptySubscription> {
    Schema::build(QueryRoot, Mutation, EmptySubscription)
        .data(DataLoader::new(TeamLoader::new(db_pool.clone())))
        .data(DataLoader::new(MatchLoader::new(db_pool.clone())))
        .extension(Tracing)
        .finish()
}
