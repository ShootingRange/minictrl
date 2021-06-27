use std::option::Option;

use crate::common::SideType;

/// Get5 Match schema
/// https://github.com/splewis/get5#match-schema
#[derive(Clone)]
pub struct Match {
    pub matchid: Option<String>,
    pub num_maps: Option<i32>,
    pub maplist: Option<Vec<String>>,
    pub skip_veto: Option<bool>,
    pub side_type: Option<SideType>,
    /// Contract: always more than 0
    pub players_per_team: Option<i32>,
    /// Contract: always 0 or more
    pub min_players_to_ready: Option<i32>,
    pub favored_percentage_team1: Option<i32>,
    pub favored_percentage_text: Option<String>,
    pub cvars: Option<Vec<String>>,
    pub spectators: Option<Spectators>,
    pub team1: Team,
    pub team2: Team,
    pub match_title: Option<String>,
}

/// Subset of Get5's Team schema, with only the fields necessary used for identifying a spectator account
#[derive(Clone)]
pub struct Spectators {
    pub name: String,
    pub players: Vec<Player>,
}

/// Get5 Team schema
#[derive(Clone)]
pub struct Team {
    pub name: String,
    pub tag: Option<String>,
    pub flag: Option<String>,
    pub logo: Option<String>,
    pub players: Vec<Player>,
    pub series_score: Option<i32>,
    pub match_text: Option<String>,
}

/// Player schema from Get5's Team schema
#[derive(Clone)]
#[allow(non_snake_case)]
pub struct Player {
    /// Contract: Is a valid steamID, steamID3 or steamID64
    pub steamID: String,
    pub name: Option<String>,
}
