use crate::common::SideType;
use crate::get5::basic::*;
use serde::de::{Unexpected, Visitor};
use serde::ser::*;
use serde::Serializer;
use serde::{Deserializer, Serialize};
use sqlx::types::ipnetwork::IpNetwork;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::net::IpAddr;
use std::str::FromStr;

impl Serialize for Spectators {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;

        map.serialize_entry("name", &self.name)?;

        let player_map: HashMap<String, String> = self
            .players
            .iter()
            .map(|player| {
                let name: String = match &player.name {
                    None => "".to_string(),
                    Some(n) => n.clone(),
                };
                (player.steamID.clone(), name)
            })
            .collect();
        map.serialize_entry("players", &player_map)?;

        map.end()
    }
}

impl Serialize for Team {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("name", &self.name)?;

        if let Some(tag) = &self.tag {
            map.serialize_entry("tag", tag)?;
        }

        if let Some(flag) = &self.flag {
            map.serialize_entry("flag", &flag)?;
        }

        if let Some(logo) = &self.logo {
            map.serialize_entry("logo", logo)?;
        }

        if !self.players.is_empty() {
            let player_map: HashMap<String, String> = self
                .players
                .iter()
                .map(|player| {
                    let name: String = match &player.name {
                        None => "".to_string(),
                        Some(n) => n.clone(),
                    };
                    (player.steamID.clone(), name)
                })
                .collect();
            map.serialize_entry("players", &player_map)?;
        }

        if let Some(series_score) = self.series_score {
            map.serialize_entry("series_score", &series_score)?;
        }

        if let Some(match_text) = &self.match_text {
            map.serialize_entry("match_text", match_text)?;
        }

        map.end()
    }
}

impl Serialize for Match {
    fn serialize<G>(&self, serializer: G) -> Result<G::Ok, G::Error>
    where
        G: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        if let Some(matchid) = &self.matchid {
            map.serialize_entry("matchid", matchid)?;
        }

        if let Some(num_maps) = self.num_maps {
            map.serialize_entry("num_maps", &num_maps)?;
        }

        if let Some(maplist) = &self.maplist {
            map.serialize_entry("maplist", maplist)?;
        }

        if let Some(skip_veto) = self.skip_veto {
            map.serialize_entry("skip_veto", &skip_veto)?;
        }

        if let Some(side_type) = &self.side_type {
            map.serialize_entry("side_type", side_type)?;
        }

        if let Some(players_per_team) = self.players_per_team {
            map.serialize_entry("players_per_team", &players_per_team)?;
        }

        if let Some(min_players_to_ready) = self.min_players_to_ready {
            map.serialize_entry("min_players_to_ready", &min_players_to_ready)?;
        }

        if let Some(favored_percentage_team1) = self.favored_percentage_team1 {
            map.serialize_entry("favored_percentage_team1", &favored_percentage_team1)?;
        }

        if let Some(favored_percentage_text) = &self.favored_percentage_text {
            map.serialize_entry("favored_percentage_text", favored_percentage_text)?;
        }

        if let Some(cvars) = &self.cvars {
            map.serialize_entry("cvars", cvars)?;
        }

        if let Some(spectators) = &self.spectators {
            map.serialize_entry("spectators", spectators)?;
        }

        map.serialize_entry("team1", &self.team1)?;

        map.serialize_entry("team2", &self.team2)?;

        if let Some(match_title) = &self.match_title {
            map.serialize_entry("match_title", match_title)?;
        }

        map.end()
    }
}

impl Serialize for SideType {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        match self {
            SideType::Standard => serializer.serialize_str("standard"),
            SideType::NeverKnife => serializer.serialize_str("never_knife"),
            SideType::AlwaysKnife => serializer.serialize_str("always_knife"),
        }
    }
}

pub(crate) fn serialize_ipnetwork<S>(addr: &IpNetwork, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(addr.ip().to_string().as_str())
}

pub(crate) fn deserialize_ipnetwork<'de, D>(d: D) -> Result<IpNetwork, D::Error>
where
    D: Deserializer<'de>,
{
    d.deserialize_str(IpNetworkVisitor {})
}

struct IpNetworkVisitor {}

impl<'de> Visitor<'de> for IpNetworkVisitor {
    type Value = IpNetwork;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("IPv4 or IPv6 address")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let addr = IpAddr::from_str(v).map_err(|err| {
            serde::de::Error::invalid_value(Unexpected::Other(err.to_string().as_str()), &self)
        })?;
        Ok(IpNetwork::from(addr))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let addr = IpAddr::from_str(v.as_str()).map_err(|err| {
            serde::de::Error::invalid_value(Unexpected::Other(err.to_string().as_str()), &self)
        })?;
        Ok(IpNetwork::from(addr))
    }
}
