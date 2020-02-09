use juniper::FieldResult;
use juniper::RootNode;
use crate::database::models::{Team, NewTeam};
use juniper::http::graphiql::graphiql_source;
use actix_web::{HttpResponse, web};
use std::sync::Arc;
use juniper::http::GraphQLRequest;

pub struct QueryRoot;

#[juniper::object]
impl QueryRoot {
    fn team(id: i32) -> FieldResult<Team> {
        Result::Ok(Team {
            id,
            name: "".to_string(),
            country: None,
            logo: None
        })
    }
}

pub struct MutationRoot;

#[juniper::object]
impl MutationRoot {
    fn createTeam(new_team: NewTeam) -> FieldResult<Team> {
        Ok(Team {
            id: 0,
            name: "".to_string(),
            country: None,
            logo: None
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
    st: web::Data<Arc<Schema>>,
    data: web::Json<GraphQLRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let user = web::block(move || {
        let res = data.execute(&st, &());
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
    })
        .await?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(user))
}
