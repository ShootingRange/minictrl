use crate::models::Side;
use std::option::Option;

/// Get5 Match schema
/// https://github.com/splewis/get5#match-schema
pub trait Match<P, S, T>
where
    P: Player,
    S: Spectator<P>,
    T: Team<P>,
{
    fn matchid(&self) -> Option<String>;
    //fn set_matchid(&self, matchid: &str);

    fn num_maps(&self) -> Option<i32>;
    //// Contract: must be a odd number more than 0
    //fn set_num_maps(&self, num_maps: i32);

    fn maplist(&self) -> Option<Vec<String>>;
    //// Contract: must be a non-empty list, and entries must be values present on the CS:GO server
    //fn set_maplist(&self, maps: &[&str]);

    fn skip_veto(&self) -> Option<bool>;

    fn side_type(&self) -> Option<Side>;

    /// Contract: always more than 0
    fn players_per_team(&self) -> Option<i32>;

    /// Contract: always 0 or more
    fn min_players_to_ready(&self) -> Option<i32>;

    fn favored_percentage_team1(&self) -> Option<i32>;

    fn favored_percentate_text(&self) -> Option<String>;

    fn cvars(&self) -> Option<Vec<String>>;

    fn spectators(&self) -> Option<Vec<S>>;

    fn team1(&self) -> T;

    fn team2(&self) -> T;

    fn match_title(&self) -> Option<String>;
}

/// Subset of Get5's Team schema, with only the fields necessary used for identifying a spectator account
pub trait Spectator<P: Player> {
    fn name(&self) -> String;

    fn players(&self) -> Vec<P>;
}

/// Get5 Team schema
pub trait Team<P: Player> {
    fn name(&self) -> String;

    fn tag(&self) -> Option<String>;

    fn flag(&self) -> Option<[char; 2]>;

    fn logo(&self) -> Option<String>;

    fn players(&self) -> Vec<P>;

    fn series_score(&self) -> Option<i32>;

    fn match_text(&self) -> Option<String>;
}

/// Player schema from Get5's Team schema
#[allow(non_snake_case)]
pub trait Player {
    /// Contract: Is a valid steamID, steamID3 or steamID64
    fn steamID(&self) -> String;

    fn name(&self) -> Option<String>;
}
