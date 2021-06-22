use crate::common::SideType;
use crate::get5::serializer::{deserialize_ipnetwork, serialize_ipnetwork};
use serde::Serialize;
use sqlx::types::ipnetwork::IpNetwork;

pub type CountryCode = String;

#[derive(Deserialize, Serialize, Debug, sqlx::FromRow)]
pub struct Team {
    pub id: i32,
    pub name: String,
    pub country: Option<CountryCode>,
    pub logo: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, sqlx::FromRow)]
pub struct Player {
    pub id: i32,
    pub name: String,
    pub team_id: i32,
    pub tag: Option<String>,
    pub steamid: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, sqlx::FromRow)]
pub struct Server {
    pub id: i32,
    #[serde(
        serialize_with = "serialize_ipnetwork",
        deserialize_with = "deserialize_ipnetwork"
    )]
    pub host: IpNetwork,
    pub port: i32,
    pub type_: Option<String>,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug, sqlx::FromRow)]
pub struct Spectator {
    pub id: i32,
    // TODO name, should we fetch this from steam?
    pub steamid: String,
}

#[derive(Serialize, Debug, sqlx::FromRow)]
pub struct Match {
    pub id: i32,
    pub server_id: i32,
    pub team1_id: i32,
    pub team2_id: i32,
    pub team1_score: Option<i32>,
    pub team2_score: Option<i32>,
    pub num_maps: i32,
    pub skip_veto: bool,
    pub veto_first: SideType,
    pub players_per_team: i32,
    pub min_player_to_ready: i32,
}

#[derive(Deserialize, Serialize, Debug, sqlx::FromRow)]
pub struct MapList {
    pub id: i32,
    pub match_id: i32,
    pub order: i32,
    pub map: String,
}

#[derive(Deserialize, Serialize, Debug, sqlx::FromRow)]
pub struct MatchSpectator {
    pub id: i32,
    pub match_id: i32,
    pub spectator_id: i32,
}
