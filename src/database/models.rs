use crate::ipnetwork::IpNetwork;
use crate::common::Side;
use crate::database::schema::*;

pub type CountryCode = String;

#[derive(Identifiable, Queryable, Serialize)]
pub struct Team {
    pub id: i32,
    pub name: String,
    pub country: Option<CountryCode>,
    pub logo: Option<String>,
}

#[derive(Insertable, GraphQLInputObject)]
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

#[derive(Insertable, GraphQLInputObject)]
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
    pub port: u16,
    pub type_: Option<String>,
}

#[derive(Identifiable, Queryable)]
pub struct Spectator {
    pub id: i32,
    pub steamid: String,
}

#[derive(Identifiable, Queryable)]
#[table_name = "matches"]
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

#[derive(Identifiable, Queryable, Associations)]
#[belongs_to(Match)]
#[table_name = "maplist"]
pub struct MapList {
    pub id: i32,
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
