use crate::common::SideType;
use async_graphql::SimpleObject;
use sqlx::types::Uuid;

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
    pub name: String,
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
