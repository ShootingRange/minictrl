use crate::common::SideType;
use crate::get5::serializer::{
    deserialize_ipnetwork, deserialize_uuid, serialize_ipnetwork, serialize_uuid,
};
use serde::Serialize;
use sqlx::types::ipnetwork::IpNetwork;
use sqlx::types::Uuid;

pub type CountryCode = String;

#[derive(Deserialize, Serialize, Debug, sqlx::FromRow)]
pub struct Team {
    #[serde(
        serialize_with = "serialize_uuid",
        deserialize_with = "deserialize_uuid"
    )]
    pub id: Uuid,
    pub name: String,
    pub country: Option<CountryCode>,
    pub logo: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, sqlx::FromRow)]
pub struct Player {
    #[serde(
        serialize_with = "serialize_uuid",
        deserialize_with = "deserialize_uuid"
    )]
    pub id: Uuid,
    pub name: String,
    #[serde(
        serialize_with = "serialize_uuid",
        deserialize_with = "deserialize_uuid"
    )]
    pub team_id: Uuid,
    pub tag: Option<String>,
    pub steamid: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, sqlx::FromRow)]
pub struct Server {
    #[serde(
        serialize_with = "serialize_uuid",
        deserialize_with = "deserialize_uuid"
    )]
    pub id: Uuid,
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
    #[serde(
        serialize_with = "serialize_uuid",
        deserialize_with = "deserialize_uuid"
    )]
    pub id: Uuid,
    // TODO name, should we fetch this from steam?
    pub steamid: String,
}

#[derive(Serialize, Debug, sqlx::FromRow)]
pub struct Match {
    #[serde(
        serialize_with = "serialize_uuid",
        deserialize_with = "deserialize_uuid"
    )]
    pub id: Uuid,
    #[serde(
        serialize_with = "serialize_uuid",
        deserialize_with = "deserialize_uuid"
    )]
    pub server_id: Uuid,
    #[serde(
        serialize_with = "serialize_uuid",
        deserialize_with = "deserialize_uuid"
    )]
    pub team1_id: Uuid,
    #[serde(
        serialize_with = "serialize_uuid",
        deserialize_with = "deserialize_uuid"
    )]
    pub team2_id: Uuid,
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
    #[serde(
        serialize_with = "serialize_uuid",
        deserialize_with = "deserialize_uuid"
    )]
    pub id: Uuid,
    #[serde(
        serialize_with = "serialize_uuid",
        deserialize_with = "deserialize_uuid"
    )]
    pub match_id: Uuid,
    pub order: i32,
    pub map: String,
}

#[derive(Deserialize, Serialize, Debug, sqlx::FromRow)]
pub struct MatchSpectator {
    #[serde(
        serialize_with = "serialize_uuid",
        deserialize_with = "deserialize_uuid"
    )]
    pub match_id: Uuid,
    #[serde(
        serialize_with = "serialize_uuid",
        deserialize_with = "deserialize_uuid"
    )]
    pub spectator_id: Uuid,
}
