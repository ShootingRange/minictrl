extern crate minictrl;

use minictrl::get5::basic::*;
use minictrl::get5::serializer::*;
use minictrl::get5::schema;
use std::marker::PhantomData;
use serde_json::Result;

fn main() -> Result<()> {
    let t1: Box<dyn schema::Team<Player>> = Box::new(Team {
        name: "Red".to_string(),
        tag: None,
        flag: None,
        logo: None,
        players: vec![],
        series_score: None,
        match_text: None
    });

    let t2: Box<dyn schema::Team<Player>> = Box::new(Team {
        name: "Blue".to_string(),
        tag: None,
        flag: None,
        logo: None,
        players: vec![],
        series_score: None,
        match_text: None
    });

    let s: Box<dyn schema::Spectator<Player>> = Box::new(Spectator {
        name: "Spectator".to_string(),
        players: vec![],
    });

    let m = Box::new(Match {
        phantom_player: PhantomData,
        matchid: Some("foobar".to_string()),

        num_maps: None,
        maplist: None,
        skip_veto: None,
        side_type: None,
        players_per_team: None,
        min_players_to_ready: None,
        favored_percentage_team1: None,
        favored_percentage_text: None,
        cvars: None,
        spectators: Some(vec![s]), // TODO should spectators be a vector?
        team1: t1,
        team2: t2,
        match_title: None
    });

    let j = serde_json::to_string(&t1)?;
    println!("{}", j);
    let j = serde_json::to_string(&t2)?;
    println!("{}", j);
    let j = serde_json::to_string(&s)?;
    println!("{}", j);

    Ok(())
}
