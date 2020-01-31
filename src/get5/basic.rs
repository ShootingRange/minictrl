use crate::models::Side;
use std::option::Option;
use crate::get5::schema;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct Match<P: schema::Player, S: schema::Spectator<P>, T: schema::Team<P>> {
    pub phantom_player: PhantomData<P>,
    pub matchid: Option<String>,
    pub num_maps: Option<i32>,
    pub maplist: Option<Vec<String>>,
    pub skip_veto: Option<bool>,
    pub side_type: Option<Side>,
    pub players_per_team: Option<i32>,
    pub min_players_to_ready: Option<i32>,
    pub favored_percentage_team1: Option<i32>,
    pub favored_percentage_text: Option<String>,
    pub cvars: Option<Vec<String>>,
    pub spectators: Option<Vec<S>>,
    pub team1: T,
    pub team2: T,
    pub match_title: Option<String>,
}

#[derive(Clone)]
pub struct Spectator<P: schema::Player> {
    pub name: String,
    pub players: Vec<P>,
}

#[derive(Clone)]
pub struct Team<P: schema::Player> {
    pub name: String,
    pub tag: Option<String>,
    pub flag: Option<[char; 2]>,
    pub logo: Option<String>,
    pub players: Vec<P>,
    pub series_score: Option<i32>,
    pub match_text: Option<String>,
}

#[derive(Clone)]
#[allow(non_snake_case)]
pub struct Player {
    pub steamID: String,
    pub name: Option<String>,
}

impl<P: schema::Player, S: schema::Spectator<P> + Clone, T: schema::Team<P> + Clone> schema::Match<P, S, T> for Match<P, S, T> {
    fn matchid(&self) -> Option<String> {
        self.matchid.clone()
    }

    fn num_maps(&self) -> Option<i32> {
        self.num_maps.clone()
    }

    fn maplist(&self) -> Option<Vec<String>> {
        self.maplist.clone()
    }

    fn skip_veto(&self) -> Option<bool> {
        self.skip_veto.clone()
    }

    fn side_type(&self) -> Option<Side> {
        self.side_type.clone()
    }

    fn players_per_team(&self) -> Option<i32> {
        self.players_per_team.clone()
    }

    fn min_players_to_ready(&self) -> Option<i32> {
        self.min_players_to_ready.clone()
    }

    fn favored_percentage_team1(&self) -> Option<i32> {
        self.favored_percentage_team1.clone()
    }

    fn favored_percentate_text(&self) -> Option<String> {
        self.favored_percentage_text.clone()
    }

    fn cvars(&self) -> Option<Vec<String>> {
        self.cvars.clone()
    }

    fn spectators(&self) -> Option<Vec<S>> {
        self.spectators.clone()
    }

    fn team1(&self) -> T {
        self.team1.clone()
    }

    fn team2(&self) -> T {
        self.team2.clone()
    }

    fn match_title(&self) -> Option<String> {
        self.match_title.clone()
    }
}

impl<P: schema::Player + Clone> schema::Spectator<P> for Spectator<P> {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn players(&self) -> Vec<P> {
        self.players.clone()
    }
}

impl<P: schema::Player + Clone> schema::Team<P> for Team<P> {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn tag(&self) -> Option<String> {
        self.tag.clone()
    }

    fn flag(&self) -> Option<[char; 2]> {
        self.flag.clone()
    }

    fn logo(&self) -> Option<String> {
        self.logo.clone()
    }

    fn players(&self) -> Vec<P> {
        self.players.clone()
    }

    fn series_score(&self) -> Option<i32> {
        self.series_score.clone()
    }

    fn match_text(&self) -> Option<String> {
        self.match_text.clone()
    }
}

#[allow(non_snake_case)]
impl schema::Player for Player {
    fn steamID(&self) -> String {
        self.steamID.clone()
    }

    fn name(&self) -> Option<String> {
        self.name.clone()
    }
}
