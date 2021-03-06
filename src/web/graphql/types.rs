use async_graphql::{InputObject, SimpleObject};
use sqlx::types::Uuid;

use crate::common::SideType;
use crate::database;

#[derive(SimpleObject)]
pub struct Team {
    pub id: Uuid,
    pub name: String,
    pub country: Option<String>,
    pub logo: Option<String>,
    pub players: Vec<Player>,
}

#[derive(SimpleObject)]
pub struct Player {
    pub steamid: String,
    pub name: Option<String>,
    pub tag: Option<String>,
}

#[derive(SimpleObject)]
pub struct Server {
    pub id: Uuid,
    pub host: String,
    pub port: i32,
    pub r#type: Option<String>,
    pub rcon_password: String,
}

impl From<crate::database::models::Server> for Server {
    fn from(server: database::models::Server) -> Self {
        Server {
            id: server.id,
            host: server.host.ip().to_string(),
            port: server.port,
            rcon_password: server.password,
            r#type: server.r#type,
        }
    }
}

#[derive(InputObject)]
pub struct ServerInput {
    pub host: String,
    pub port: i32,
    pub r#type: Option<String>,
    pub rcon_password: String,
}

#[derive(SimpleObject)]
pub struct Match {
    pub id: Uuid,
    pub server: Option<Server>,
    pub team1: Team,
    pub team2: Team,
    pub team1_score: Option<i32>,
    pub team2_score: Option<i32>,
    pub num_maps: i32,
    pub skip_veto: bool,
    pub veto_first: SideType,
    pub players_per_team: i32,
    pub min_player_to_ready: i32,
    pub maps: Vec<String>,
    pub spectators: Vec<Spectator>,
}

#[derive(SimpleObject)]
pub struct Spectator {
    pub steamid: String,
    // Note: Steam profile might be hidden which prevents us from fetching the name of the steamid
    pub name: Option<String>,
}
