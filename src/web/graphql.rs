use juniper::FieldResult;
use juniper::RootNode;
use crate::database::models::{Team, NewTeam, Player, NewPlayer};
use juniper::http::graphiql::graphiql_source;
use actix_web::{HttpResponse, web};
use juniper::http::GraphQLRequest;
use actix::{Addr, MailboxError};
use crate::actors::database::DbExecutor;
use std::error::Error;

pub struct Context {
    db: Addr<DbExecutor>
}

impl juniper::Context for Context {}

pub struct QueryRoot;

#[juniper::object]
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

    fn players(&self) -> Vec<Player> {
        vec![]
    }
}

#[juniper::object(
    Context = Context,
)]
impl QueryRoot {
    async fn team(context: &Context, id: i32) -> FieldResult<Team> {
        let team = context.db.send(crate::actors::database::team::FindTeamById {
            id
        }).await;

        match team {
            Ok(team_result) => {
                match team_result {
                    Ok(team) => Result::ok(team),
                    Err(err) => FieldResult::err(err),
                }
            },
            Err(err) => FieldResult::err(err),
        }
    }

    /*fn player(id: i32) -> FieldResult<GraphQLPlayer> {
        Ok(GraphQLPlayer {

        })
    }*/
}

pub struct MutationRoot;

#[juniper::object(
    Context = Context,
)]
impl MutationRoot {
    fn createTeam(team: NewTeam, players: Option<Vec<NewPlayer>>) -> FieldResult<Team> {
        Ok(Team {
            id: 0,
            name: "".to_string(),
            country: None,
            logo: None
        })
    }

    fn updateTeam(id: i32, team: NewTeam) -> FieldResult<Team> {
        Ok(Team {
            id: 0,
            name: "".to_string(),
            country: None,
            logo: None
        })
    }

    fn deleteTeam(id: i32) -> FieldResult<Team> {
        Ok(Team {
            id: 0,
            name: "".to_string(),
            country: None,
            logo: None
        })
    }

    fn addPlayer(new_team: NewPlayer) -> FieldResult<Player> {
        Ok(Player {
            id: 0,
            team_id: 0,
            name: "".to_string(),
            tag: None,
            steamid: None
        })
    }

    fn removePlayer(id: i32) -> FieldResult<Player> {
        Ok(Player {
            id: 0,
            team_id: 0,
            name: "".to_string(),
            tag: None,
            steamid: None
        })
    }

    fn updatePlayer(id: i32, player: NewPlayer) -> FieldResult<Player> {
        Ok(Player {
            id: 0,
            team_id: 0,
            name: "".to_string(),
            tag: None,
            steamid: None
        })
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
    db: web::Data<Addr<DbExecutor>>
) -> Result<HttpResponse, actix_web::Error> {
    let user = web::block(move || {
        let res = data.execute_async::<Context, QueryRoot, MutationRoot>(&st, &Context{
            db: db.get_ref().clone(),
        });
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
    })
        .await?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(user))
}
