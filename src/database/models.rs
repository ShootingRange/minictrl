use crate::common::SideType;
use crate::database::schema::*;
use ipnetwork::IpNetwork;

pub type CountryCode = String;

#[derive(Identifiable, Queryable, Serialize)]
pub struct Team {
    pub id: i32,
    pub name: String,
    pub country: Option<CountryCode>,
    pub logo: Option<String>,
}

#[derive(Insertable)]
#[table_name = "teams"]
pub struct NewTeam {
    pub name: String,
    pub country: Option<CountryCode>,
    pub logo: Option<String>,
}

#[derive(Identifiable, Queryable, Serialize)]
pub struct Player {
    pub id: i32,
    pub name: String,
    pub team_id: i32,
    pub tag: Option<String>,
    pub steamid: Option<String>,
}

#[derive(Insertable)]
#[table_name = "players"]
pub struct NewPlayer {
    pub team_id: i32,
    pub name: String,
    pub tag: Option<String>,
    pub steamid: Option<String>,
}

#[derive(Identifiable, Queryable)]
pub struct Server {
    pub id: i32,
    pub host: IpNetwork,
    pub port: i32,
    pub type_: Option<String>,
}

#[derive(Insertable)]
#[table_name = "servers"]
pub struct NewServer {
    pub host: IpNetwork,
    pub port: i32,
    pub type_: Option<String>,
}

#[derive(Identifiable, Queryable)]
pub struct Spectator {
    pub id: i32,
    // TODO name, should we fetch this from steam?
    pub steamid: String,
}

#[derive(Insertable)]
#[table_name = "spectators"]
pub struct NewSpectator {
    pub steamid: String,
}

#[derive(Identifiable, Queryable)]
#[table_name = "matches"]
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

#[derive(Insertable)]
#[table_name = "matches"]
pub struct NewMatch {
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

#[derive(Identifiable, Queryable, Associations)]
#[belongs_to(Match)]
#[table_name = "maplist"]
pub struct MapList {
    pub id: i32,
    pub match_id: i32,
    pub order: i32,
    pub map: String,
}

#[derive(Insertable)]
#[table_name = "maplist"]
pub struct NewMapList {
    pub match_id: i32,
    pub order: i32,
    pub map: String,
}

#[derive(Identifiable, Queryable, Associations)]
#[belongs_to(Match)]
#[table_name = "match_spectator"]
pub struct MatchSpectator {
    pub id: i32,
    pub match_id: i32,
    pub spectator_id: i32,
}

#[derive(Insertable)]
#[table_name = "match_spectator"]
pub struct NewMatchSpectator {
    pub match_id: i32,
    pub spectator_id: i32,
}
