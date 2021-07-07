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

pub(crate) struct Query;

#[async_graphql::Object]
impl Query {
    async fn teams(&self, ctx: &Context<'_>, ids: Vec<Uuid>) -> async_graphql::Result<Vec<Team>> {
        let mut teams_raw = ctx
            .data_unchecked::<DataLoader<TeamLoader>>()
            .load_many(ids)
            .await?;

        let team_ids = teams_raw.keys().cloned().collect::<Vec<Uuid>>();

        let mut players_by_team = ctx
            .data_unchecked::<DataLoader<PlayerTeamLoader>>()
            .load_many(team_ids)
            .await?;

        let teams = teams_raw
            .drain()
            .map(|(_key, team)| {
                let players = if ctx.look_ahead().field("players").exists() {
                    players_by_team
                        .remove(&team.id)
                        .expect(format!("missing player for team (id={})", team.id).as_str())
                        .drain(..)
                        .map(|player| Player {
                            steamid: player.steamid,
                            name: player.name,
                            tag: player.tag,
                        })
                        .collect()
                } else {
                    vec![]
                };

                Team {
                    id: team.id,
                    name: team.name,
                    country: team.country,
                    logo: team.logo,
                    players,
                }
            })
            .collect();

        Ok(teams)
    }

    async fn players(
        &self,
        ctx: &Context<'_>,
        ids: Vec<Uuid>,
    ) -> async_graphql::Result<Vec<Player>> {
        todo!()
    }

    async fn servers(
        &self,
        ctx: &Context<'_>,
        ids: Vec<Uuid>,
    ) -> async_graphql::Result<Vec<Server>> {
        todo!()
    }

    async fn spectators(
        &self,
        ctx: &Context<'_>,
        ids: Vec<Uuid>,
    ) -> async_graphql::Result<Vec<Spectator>> {
        todo!()
    }

    async fn r#match(
        &self,
        ctx: &Context<'_>,
        ids: Vec<Uuid>,
    ) -> async_graphql::Result<Vec<r#Match>> {
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

    async fn create_server(&self, server: ServerInput) -> async_graphql::Result<Server> {
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

pub(crate) fn init_schema(db_pool: Pool<Postgres>) -> Schema<Query, Mutation, EmptySubscription> {
    Schema::build(Query, Mutation, EmptySubscription)
        .data(DataLoader::new(TeamLoader::new(db_pool.clone())))
        .data(DataLoader::new(MatchLoader::new(db_pool.clone())))
        .data(DataLoader::new(PlayerLoader::new(db_pool.clone())))
        .data(DataLoader::new(PlayerTeamLoader::new(db_pool.clone())))
        .extension(Tracing)
        .finish()
}
