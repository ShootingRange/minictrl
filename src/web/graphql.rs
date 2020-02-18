use juniper::{FieldResult, FieldError, ScalarValue};
use juniper::RootNode;
use crate::database::models::{Team, NewTeam, Player, NewPlayer};
use juniper::http::graphiql::graphiql_source;
use actix_web::{HttpResponse, web};
use juniper::http::GraphQLRequest;
use actix::Addr;
use crate::actors::database::DbExecutor;

pub struct Context {
    db: Addr<DbExecutor>
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
        let players = context.db.send(crate::actors::database::player::FindPlayersByTeamId {
            team_id: self.id
        }).await;

        match players {
            Ok(result) => {
                match result {
                    Ok(ps) => FieldResult::Ok(ps),
                    Err(err) => FieldResult::Err(FieldError::from(err)),
                }
            }
            Err(err) => FieldResult::Err(FieldError::from(err)),
        }
    }
}

#[juniper::graphql_object(
    Context = Context,
)]
impl QueryRoot {
    async fn team(context: &Context, id: i32) -> FieldResult<Team> {
        let team = context.db.send(crate::actors::database::team::FindTeamById {
            id
        }).await;

        match team {
            Ok(result) => {
                match result {
                    Ok(team) => FieldResult::Ok(team),
                    Err(err) => FieldResult::Err(FieldError::from(err)),
                }
            }
            Err(err) => {
                if cfg!(debug_assertions) {
                    FieldResult::Err(FieldError::from(err))
                } else {
                    FieldResult::Err(FieldError::from("Over capacity, try again later"))
                }
            },
        }
    }

    async fn player(context: &Context, id: i32) -> FieldResult<Player> {
        let player = context.db.send(crate::actors::database::player::FindPlayerById {
            id
        }).await;

        match player {
            Ok(result) => {
                match result {
                    Ok(team) => FieldResult::Ok(team),
                    Err(err) => FieldResult::Err(FieldError::from(err)),
                }
            }
            Err(err) => {
                if cfg!(debug_assertions) {
                    FieldResult::Err(FieldError::from(err))
                } else {
                    FieldResult::Err(FieldError::from("Over capacity, try again later"))
                }
            },
        }
    }
}

pub struct MutationRoot;

#[juniper::graphql_object(
    Context = Context,
)]
impl MutationRoot {
    /*
    fn createTeam(team: NewTeam, players: Option<Vec<NewPlayer>>) -> FieldResult<Team> {
        unimplemented!()
    }

    fn updateTeam(id: i32, team: NewTeam) -> FieldResult<Team> {
        unimplemented!()
    }

    fn deleteTeam(id: i32) -> FieldResult<Team> {
        unimplemented!()
    }

    fn addPlayer(new_team: NewPlayer) -> FieldResult<Player> {
        unimplemented!()
    }

    fn removePlayer(id: i32) -> FieldResult<Player> {
        unimplemented!()
    }

    fn updatePlayer(id: i32, player: NewPlayer) -> FieldResult<Player> {
        unimplemented!()
    }
    */
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
    let context = Context{
        db: db.get_ref().clone(),
    };
    let res = data.execute_async::<Context, QueryRoot, MutationRoot>(&st, &context).await;
    let user = serde_json::to_string(&res);
    match user {
        Ok(json) => {
            Ok(HttpResponse::Ok()
                .content_type("application/json")
                .body(json))
        },
        Err(err) => std::result::Result::Err(actix_web::Error::from(err)),
    }
}
