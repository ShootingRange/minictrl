use crate::ipnetwork::IpNetwork;

#[derive(Queryable)]
pub struct Team {
    pub id: i32,
    pub name: String,
    pub countr: [char; 2],
    pub logo: String,
}

#[derive(Queryable)]
pub struct Player {
    pub id: i32,
    pub team_id: i32,
    pub name: String,
    pub tag: String,
    pub steamid: String,
}

#[derive(Queryable)]
pub struct Server {
    pub id: i32,
    pub host: IpNetwork,
    pub port: u16,
    pub type_: String,
}

#[derive(Queryable)]
pub struct Spectator {
    pub id: i32,
    pub steamid: String,
}

#[derive(Clone)]
pub enum Side {
    Standard,
    NeverKnife,
    AlwaysKnife,
}

#[derive(Queryable)]
pub struct Match {
    pub id: i32,
    pub server_id: i32,
    pub team1_id: i32,
    pub team1_score: Option<i32>,
    pub team2_score: Option<i32>,
    pub num_maps: i32,
    pub skip_veto: bool,
    pub veto_first: Side,
    pub players_per_team: i32,
    pub min_player_to_ready: i32,
}

#[derive(Queryable)]
pub struct MapList {
    pub id: i32,
    pub order: i32,
    pub map: String,
}

#[derive(Queryable)]
pub struct MatchSpectator {
    pub match_id: i32,
    pub spectator_id: i32,
}
