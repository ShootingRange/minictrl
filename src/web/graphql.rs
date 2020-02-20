use crate::actors::database::player::{CreatePlayer, DeletePlayerById};
use crate::actors::database::server::CreateServer;
use crate::actors::database::team::{CreateTeam, DeleteTeamById};
use crate::actors::database::*;
use crate::database::models::{CountryCode, Player, Server, Team};
use actix::{Addr, MailboxError};
use actix_web::{web, HttpResponse};
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use juniper::RootNode;
use juniper::{FieldError, FieldResult};
use std::net::IpAddr;
use std::str::FromStr;

pub struct Context {
    db: Addr<DbExecutor>,
}

fn unpack_dbexecutor<T>(resp: Result<Result<T, DbActorError>, MailboxError>) -> FieldResult<T> {
    match resp {
        Ok(result) => match result {
            Ok(item) => FieldResult::Ok(item),
            Err(err) => FieldResult::Err(FieldError::from(err)),
        },
        Err(err) => {
            if cfg!(debug_assertions) {
                FieldResult::Err(FieldError::from(err))
            } else {
                FieldResult::Err(FieldError::from("Over capacity, try again later"))
            }
        }
    }
}

impl juniper::Context for Context {}

pub struct QueryRoot;

#[juniper::graphql_object(
Context = Context
)]
impl Team {
    fn id(&self) -> i32 {
        self.id
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn country(&self) -> Option<String> {
        self.country.clone()
    }

    fn tag(&self) -> Option<String> {
        self.logo.clone()
    }

    async fn players(&self, context: &Context) -> FieldResult<Vec<Player>> {
        let players = context
            .db
            .send(player::FindPlayersByTeamId { team_id: self.id })
            .await;

        unpack_dbexecutor(players)
    }
}

#[juniper::graphql_object(
Context = Context
)]
impl Player {
    fn id(&self) -> i32 {
        self.id
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn team_id(&self) -> i32 {
        self.team_id
    }

    fn tag(&self) -> Option<String> {
        self.tag.clone()
    }

    fn steamid(&self) -> Option<String> {
        self.steamid.clone()
    }

    async fn team(&self, context: &Context) -> FieldResult<Team> {
        let team = context
            .db
            .send(team::FindTeamById { id: self.team_id })
            .await;

        unpack_dbexecutor(team)
    }
}

#[juniper::graphql_object(
Context = Context
)]
impl Server {
    fn id(&self) -> i32 {
        self.id
    }

    fn host(&self) -> String {
        self.host.ip().to_string()
    }

    fn port(&self) -> i32 {
        self.port
    }

    fn type_(&self) -> Option<String> {
        self.type_.clone()
    }
}

#[juniper::graphql_object(
Context = Context,
)]
impl QueryRoot {
    async fn team(context: &Context, id: i32) -> FieldResult<Team> {
        let team = context.db.send(team::FindTeamById { id }).await;

        unpack_dbexecutor(team)
    }

    async fn teams(context: &Context) -> FieldResult<Vec<Team>> {
        let teams = context.db.send(team::GetTeams {}).await;

        unpack_dbexecutor(teams)
    }

    async fn player(context: &Context, id: i32) -> FieldResult<Player> {
        let player = context.db.send(player::FindPlayerById { id }).await;

        unpack_dbexecutor(player)
    }

    async fn server(context: &Context, id: i32) -> FieldResult<Server> {
        let server = context.db.send(player::FindServerById { id }).await;

        unpack_dbexecutor(server)
    }
}

pub struct MutationRoot;

#[juniper::graphql_object(
Context = Context,
)]
impl MutationRoot {
    async fn createTeam(
        name: String,
        country: Option<CountryCode>,
        logo: Option<String>,
        context: &Context,
    ) -> FieldResult<Team> {
        let team = context
            .db
            .send(CreateTeam {
                name,
                country,
                logo,
            })
            .await;

        unpack_dbexecutor(team)
    }

    /*
    fn updateTeam(id: i32, team: NewTeam) -> FieldResult<Team> {
        unimplemented!()
    }
    */

    async fn deleteTeam(id: i32, context: &Context) -> FieldResult<bool> {
        let result = context.db.send(DeleteTeamById { id }).await;

        unpack_dbexecutor(result)
    }

    async fn createPlayer(
        team_id: i32,
        name: String,
        tag: Option<String>,
        steamid: Option<String>,
        context: &Context,
    ) -> FieldResult<Player> {
        let result = context
            .db
            .send(CreatePlayer {
                team_id,
                name,
                tag,
                steamid,
            })
            .await;

        unpack_dbexecutor(result)
    }

    async fn deletePlayer(id: i32, context: &Context) -> FieldResult<bool> {
        let result = context.db.send(DeletePlayerById { id }).await;

        unpack_dbexecutor(result)
    }

    /*
    fn updatePlayer(id: i32, player: NewPlayer) -> FieldResult<Player> {
        unimplemented!()
    }
    */

    async fn createServer(
        host: String,
        port: i32,
        type_: Option<String>,
        context: &Context,
    ) -> FieldResult<Server> {
        let host = match IpAddr::from_str(host.as_str()) {
            Ok(ipaddr) => ipaddr,
            Err(err) => return if cfg!(debug_assertions) {
                FieldResult::Err(FieldError::from(err))
            } else {
                FieldResult::Err(FieldError::from("Invalid IP address"))
            },
        };

        if port < 1 || port >= 65536 {
            return FieldResult::Err(FieldError::from("Invalid port, must be between 1 and 65535"))
        }

        let server = context
            .db
            .send(CreateServer {
                host,
                port: port as u16,
                r#type: type_,
            })
            .await;

        unpack_dbexecutor(server)
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {})
}

pub async fn graphiql() -> HttpResponse {
    let html = graphiql_source("http://127.0.0.1:8080/graphql");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

pub async fn graphql(
    st: web::Data<Schema>,
    data: web::Json<GraphQLRequest>,
    db: web::Data<Addr<DbExecutor>>,
) -> Result<HttpResponse, actix_web::Error> {
    let context = Context {
        db: db.get_ref().clone(),
    };
    let res = data
        .execute_async::<Context, QueryRoot, MutationRoot>(&st, &context)
        .await;
    let user = serde_json::to_string(&res);
    match user {
        Ok(json) => Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(json)),
        Err(err) => std::result::Result::Err(actix_web::Error::from(err)),
    }
}
