use std::marker::PhantomData;

use regex::{Regex, RegexSet, Captures};
use std::str::FromStr;
use std::fmt::Debug;

pub struct LogPrefix {
    pub month: i32,
    pub day: i32,
    pub year: i32,
    pub hour: i32,
    pub minute: i32,
    pub second: i32,
}

pub enum TeamAll {
    TERRORIST,
    CT,
    UNASSIGNED,
    SPECTATOR,
    CONSOLE,
}

pub enum Team {
    TERRORIST,
    CT,
}

pub enum PlayerID {
    STAMID(String),
    BOT,
    CONSOLE,
}

/// Player description as seen in the CS:GO logs.
///
/// Regex:
/// ```regex
/// "(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>"
/// ```
///
/// Constructed examples:
/// ```plain
/// alice<10><STEAM_1:0:536763442><TERRORIST>
/// bob<4><STEAM_1:0:145932671><CT>
/// ```
pub struct Player {
    pub nick: String,
    pub entity_index: i32,
    pub id: PlayerID,
    pub team: TeamAll,
}

pub enum LogEntry {
    /// Start of log file.
    LogFileStart {
        prefix: LogPrefix,
        file: String,
        game: String,
        version: i32,
    },
    /// End of log file.
    LogFileClosed {
        prefix: LogPrefix,
    },
    /// World triggered game event.
    WorldTriggeredEvent {
        prefix: LogPrefix,
        event: String,
    },
    /// World triggered game event in relation to map.
    /// Only seen with event "Match_Start".
    WorldTriggeredEventMap {
        prefix: LogPrefix,
        event: String,
        map: String,
    },
    /// World triggered event with meta information about team scores.
    /// Only seen with event "SFUI_Notice_Round_Draw".
    WorldTriggeredEventScore {
        prefix: LogPrefix,
        event: String,
        ct_score: i32,
        t_score: i32,
    },
    /// Player triggered game event.
    PlayerTriggeredEvent {
        prefix: LogPrefix,
        player: Player,
    },
    /// Team triggered game event with meta information about team scores.
    TeamTriggeredEventScore {
        prefix: LogPrefix,
        team: Team,
        event: String,
        ct_score: i32,
        t_score: i32,
    },
    /// Loading map.
    LoadingMap {
        prefix: LogPrefix,
        map: String,
    },
    /// Server dumped all it's cvars during startup.
    CvarDump {
        start: LogPrefix,
        end: LogPrefix,
        cvars: Vec<(String, String)>,
    },
    /// Started map.
    StartedMap {
        prefix: LogPrefix,
        map: String,
        crc: String,
    },
    /// Server emitted a single cvar.
    Cvar {
        prefix: LogPrefix,
        key: String,
        value: String,
    },
    /// Player entered the game.
    /// Team field is always empty.
    PlayerEnteredGame {
        prefix: LogPrefix,
        player: Player,
    },
    /// Get5 event encoded as JSON.
    Get5Event {
        prefix: LogPrefix,
        json: String,
    },
    /// Command was executed over RCON.
    /// The command can contain double quotes, take care when editing the regex.
    RconCommand {
        prefix: LogPrefix,
        client_address: String,
    },
    /// Bad password during RCON authentication.
    RconBadPassword {
        prefix: LogPrefix,
        client_address: String,
    },
    /// Player switched from on team/side to another.
    SwitchedTeam {
        prefix: LogPrefix,
        from: TeamAll,
        to: TeamAll,
    },
    /// Player picked up instrument/equipment.
    PlayerPickedUp {
        prefix: LogPrefix,
        player: Player,
        instrument: String,
    },
    /// Player dropped instrument/equipment.
    PlayerDropped {
        prefix: LogPrefix,
        player: Player,
        instrument: String,
    },
}

lazy_static! {
    /// Prefix for a CS:GO log line.
    static ref LOG_PREFIX: String = r"^L (?P<log_month>\d\d)/(?P<log_day>\d\d)/(?P<log_year>\d\d\d\d) - (?P<log_hour>\d\d):(?P<log_minute>\d\d):(?P<log_second>\d\d): ".to_string();

    static ref REGEX_STRINGS: [String; 53] = [
        LOG_PREFIX.clone() + r#"Log file started \(file "(?P<file>[^"]*)"\) \(game "(?P<game>[^"]*)"\) \(version "(?P<version>\d+)"\)$"#,
        LOG_PREFIX.clone() + r#"Log file closed$"#,
        LOG_PREFIX.clone() + r#"World triggered "(?P<event>[^"]*)"$"#,
        LOG_PREFIX.clone() + r#"World triggered "(?P<event>[^"]*)" on "(?P<map>[^"]*)"$"#,
        LOG_PREFIX.clone() + r#"World triggered "(?P<event>[^"]*)" \(CT "(?P<ct>\d+)"\) \(T "(?P<t>\d+)"\)$"#,
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" triggered "(?P<event>[^"]*)"$"#,
        LOG_PREFIX.clone() + r#"Team "(?P<team>(TERRORIST|CT))" triggered "(?P<event>[^"]*)" \(CT "(?P<ct>\d+)"\) \(T "(?P<t>\d+)"\)$"#,
        LOG_PREFIX.clone() + r#"Loading map "(?P<map>[^"]*)"$"#,
        // Server started dumping cvars.
        LOG_PREFIX.clone() + r#"server cvars start$"#,
        // Individual cvar from cvars dump.
        LOG_PREFIX.clone() + r#""(?P<cvar_key>[^"]*)" = "(?P<cvar_value>[^"]*)"$"#,
        // Server ended cvars dump.
        LOG_PREFIX.clone() + r#"server cvars end$"#,
        LOG_PREFIX.clone() + r#"Started map "(?P<map>[^"]*)" \(CRC "(?P<crc>-?\d+)"\)$"#,
        LOG_PREFIX.clone() + r#"server_cvar: "(?P<cvar_key>[^"]*)" "(?P<cvar_value>[^"]*)"$"#,
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" entered the game$"#,
        LOG_PREFIX.clone() + r#"get5_event: (?P<json>.+)$"#,
        LOG_PREFIX.clone() + r#"rcon from "(?P<ip_addres>\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}:\d{1,5})": command "(?P<command>.*)"$"#,
        LOG_PREFIX.clone() + r#"rcon from "(?P<ip_addres>\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}:\d{1,5})": Bad Password$"#,
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT))>" switched from team <(?P<from_side>(Unassigned|TERRORIST|CT|Spectator)?)> to <(?P<to_side>(Unassigned|TERRORIST|CT|Spectator)?)>$"#,
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" picked up "(?P<instrument>[^"]*)"$"#,
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" dropped "(?P<instrument>[^"]*)"$"#,
        // TODO description
        LOG_PREFIX.clone() + r#"Team playing "(?P<side>(CT|TERRORIST))": (\[(?P<readiness>(NOT )?READY)\] )?(?P<team>.*)$"#,
        // Freeze period started.
        LOG_PREFIX.clone() + r#"Starting Freeze period$"#,
        // Player left buyzone, and can no longer buy equipment until next round.
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" left buyzone with \[ (?P<instruments>([A-Za-z0-9_]*(\(\d+\))? )*)\]$"#,
        // Player send message in team chat.
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" say_team "(?P<msg>.*)"$"#,
        // Player's money changed.
        // Can be caused by round change, or purchase. It's not known what the "tracked" attribute indicates. The resulting money may be capped by `mp_maxmoney`.
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" money change (?P<money_prev>\d+)(?P<moeny_op>[\+-])(?P<money_diff>\d+) = \$(?P<money_after>\d+)( \((?P<tracked>tracked)\)( \(purchase: (?P<instrument>[A-Za-z0-9_]*(\(\d+\))?)\))?)?$"#,
        // Player purchased instrument/equipment.
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" purchased "(?P<instrument>[A-Za-z0-9_]*(\(\d+\))?)"$"#,
        // Player threw flashbang.
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" threw flashbang \[(?P<loc_x>-?\d+) (?P<loc_y>-?\d+) (?P<loc_z>-?\d+)\] flashbang entindex (?P<entindex>\d+)\)$"#, // Extra bracket at end
        // Player was blinded by flashbang thrown by another player
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" blinded for \d+\.\d{2} by "(?P<p2_nick>[^<]*)<(?P<p2_entindex>\d+)><(?P<p2_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<p2_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" from flashbang entindex (?P<entindex>\d+) $"#, // Trailing space
        // Player send message in global chat. Both teams will see these messages.
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" say "(?P<msg>.*)"$"#, // Message can contain double quotes
        // Player killed entity
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" \[(?P<loc_x>-?\d+) (?P<loc_y>-?\d+) (?P<loc_z>-?\d+)\] killed other "[^<]*<\d+>" \[(?P<ent_x>-?\d+) (?P<ent_y>-?\d+) (?P<ent_z>-?\d+)\] with "(?P<instrument>[A-Za-z0-9_]*(\(\d+\))?)"( \((?P<kill_attributes>(headshot|penetrated|headshot penetrated))\))?$"#,
        // Player killed another player with instrument
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" \[(?P<loc_x>-?\d+) (?P<loc_y>-?\d+) (?P<loc_z>-?\d+)\] killed "(?P<p2_nick>[^<]*)<(?P<p2_entindex>\d+)><(?P<p2_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<p2_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" \[(?P<p2_x>-?\d+) (?P<p2_y>-?\d+) (?P<p2_z>-?\d+)\] with "(?P<instrument>[A-Za-z0-9_]*(\(\d+\))?)"( \((?P<kill_attributes>(headshot|penetrated|headshot penetrated))\))?$"#,
        // Player threw smokegrenade
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" threw smokegrenade \[(?P<loc_x>-?\d+) (?P<loc_y>-?\d+) (?P<loc_z>-?\d+)\]$"#,
        // Player threw high explosive grenade.
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" threw hegrenade \[(?P<loc_x>-?\d+) (?P<loc_y>-?\d+) (?P<loc_z>-?\d+)\]$"#,
        // Player attacked another player
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" \[(?P<p1_x>-?\d+) (?P<p1_y>-?\d+) (?P<p1_z>-?\d+)\] attacked "(?P<player_attacked_nick>[^<]*)<(?P<player_attacked_entindex>\d+)><(?P<player_attacked_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_attacked_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" \[(?P<p2_x>-?\d+) (?P<p2_y>-?\d+) (?P<p2_z>-?\d+)\] with "(?P<instrument>[A-Za-z0-9_]*(\(\d+\))?)" \(damage "(?P<damage>\d+)"\) \(damage_armor "(?P<damage_armor>\d+)"\) \(health "(?P<health>\d+)"\) \(armor "(?P<armor>\d+)"\) \(hitgroup "(?P<hitgroup>(chest|generic|head|left arm|left leg|neck|right arm|right leg|stomach))"\)$"#,
        // Player disconnected from game server.
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" disconnected \(reason "(?P<reason>[^"]*)"\)$"#,
        // Player assisted another player in killing a third player.
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" assisted killing "(?P<player_killed_nick>[^<]*)<(?P<player_killed_entindex>\d+)><(?P<player_killed_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_killed_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>"$"#,
        // Player assisted another player in killing a third player by blinding them.
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" flash-assisted killing "(?P<player_killed_nick>[^<]*)<(?P<player_killed_entindex>\d+)><(?P<player_killed_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_killed_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>"$"#,
        // Molotov projectile spawned
        LOG_PREFIX.clone() + r#"Molotov projectile spawned at (?P<loc_x>-?\d+\.\d+) (?P<loc_y>-?\d+\.\d+) (?P<loc_z>-?\d+\.\d+), velocity (?P<vec_x>-?\d+\.\d+) (?P<vec_y>-?\d+\.\d+) (?P<vec_z>-?\d+\.\d+)$"#,
        // Player threw molotov
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" threw molotov \[(?P<loc_x>-?\d+) (?P<loc_y>-?\d+) (?P<loc_z>-?\d+)\]$"#,
        // Player conncted to game server.
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" connected, address "(?P<ip_address>[^"]*)"$"#,
        // SteamID of player was validated.
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" STEAM USERID validated$"#,
        // Team ended match with given score, and number of player participating.
        LOG_PREFIX.clone() + r#"Team "(?P<side>(CT|TERRORIST))" scored "(?P<score>\d+)" with "(?P<player_count>\d+)" players$"#,
        // Player threw decoy.
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" threw decoy \[(?P<loc_x>-?\d+) (?P<loc_y>-?\d+) (?P<loc_z>-?\d+)\]$"#,
        // Match paused.
        LOG_PREFIX.clone() + r#"Match pause is disabled - mp_unpause_match$"#,
        // Match pause ended.
        LOG_PREFIX.clone() + r#"Match pause is enabled - mp_pause_match$"#,
        // Player was killed by bomb
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" \[(?P<loc_x>-?\d+) (?P<loc_y>-?\d+) (?P<loc_z>-?\d+)\] was killed by the bomb\.$"#,
        // TODO description
        LOG_PREFIX.clone() + r#"ACCOLADE, FINAL: \{[^\}]*\},\s+[^<]*<\d+>,\s+VALUE: \d+\.\d+,\s+POS: \d+,\s+SCORE: \d+\.\d+$"#,
        // Game ended.
        // CT and T score might be swapped, this needs validation.
        LOG_PREFIX.clone() + r#"Game Over: (?P<mode>[A-Za-z0-9_]+) (?P<map_group>[A-Za-z0-9_]+) (?P<map>[A-Za-z0-9_]+) score (?P<ct_score>\d+):(?P<t_score>\d+) after (?P<time>\d+) min$"#,
        // Player changed their nickname.
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" changed name to "(?P<new_nick>[^"]*)"$"#,
        // Player commited suicide with instrument
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" \[(?P<loc_x>-?\d+) (?P<loc_y>-?\d+) (?P<loc_z>-?\d+)\] committed suicide with "(?P<instrument>[^"]*)"$"#,
        // TODO description
        LOG_PREFIX.clone() + r#"server_message: "(?P<msg>[^"]*)"$"#,
        // Failed to validate user authentication ticket.
        // Error codes are described in the [`steam_api.h` documentation](https://partner.steamgames.com/doc/api/steam_api#EAuthSessionResponse).
        LOG_PREFIX.clone() + r#"STEAMAUTH: Client (?P<player_nick>.*) received failure code (?P<code>\d+)$"#,
        // META mod has loaded plugins.
        LOG_PREFIX.clone() + r#"\[META\] Loaded (?P<plugins_loaded>\d+) plugin(s|\.)( \((?P<plugins_preloaded>\d+) already loaded\))?$"#,
    ];

    static ref REGEX: RegexSet = RegexSet::new(REGEX_STRINGS.iter()).unwrap();

    static ref SINGLE_REGEXES: Vec<Regex> = REGEX_STRINGS.iter()
        .enumerate()
        .map(|(i, regex_string)| {
            Regex::new(regex_string)
                .expect(format!("Failed create a regex for log string regex {}", i).as_str())
        })
        .collect();
}

#[async_trait]
pub trait LogEntryReader<E> {
    async fn read_log_line(self) -> Result<String, E>;
}

pub struct LogProcessor<R: LogEntryReader<E>, E> {
    _phantom: PhantomData<E>,
    reader: R,
    reading_cvar_dump: bool,
}

#[derive(Debug)]
pub enum Error<E> {
    ReaderError(E),
    Ambiguous,
    ParseError,
    Unknown(String),
}

fn extract_parse<E: FromStr>(captures: &Captures, group: &str) -> E
    where E::Err: Debug {
    // TODO build expect messages at compile time using something like the concat! macro. A combination of macros and functions could be effective, https://godbolt.org/z/bAJUG9
    captures.name(group)
        .expect(format!("no match for capture \"{}\"", group).as_str())
        .as_str()
        .parse()
        .unwrap()
        //.expect(format!("Failed to parse \"{}\"", group).as_str())
}

fn extract_into<'t, E: From<&'t str>>(captures: &Captures<'t>, group: &str) -> E {
    captures.name(group)
        .expect(format!("no match for capture \"{}\"", group).as_str())
        .as_str()
        .into()
}

fn extract_str<'t>(captures: &Captures<'t>, group: &str) -> &'t str {
    captures.name(group)
        .expect(format!("no match for capture \"{}\"", group).as_str())
        .as_str()
}

fn extract_prefix(captures: &Captures) -> LogPrefix {
    LogPrefix {
        month: extract_parse(captures, "log_month"),
        day: extract_parse(captures, "log_day"),
        year: extract_parse(captures, "log_year"),
        hour: extract_parse(captures, "log_hour"),
        minute: extract_parse(captures, "log_minute"),
        second: extract_parse(captures, "log_second"),
    }
}

impl<R: LogEntryReader<E>, E> LogProcessor<R, E> {
    pub fn new(reader: R) -> Self {
        LogProcessor {
            _phantom: Default::default(),
            reader,
            reading_cvar_dump: false,
        }
    }

    pub async fn read_entry(self) -> Result<LogEntry, Error<E>> {
        let line = match self.reader.read_log_line().await {
            Ok(line) => line,
            Err(err) => return Err(Error::ReaderError(err)),
        };

        let matchs: Vec<usize> = REGEX.matches(&line)
            .iter()
            .collect();

        let index = match matchs.len() {
            // No regex matched the log line
            0 => return Result::Err(Error::Unknown(line)),
            // Success
            1 => matchs[0],
            // If more than one regex matches the log line can't parsed decidedly
            // ASSUMPTION: usize can't represent a number less than zero
            _ => return Result::Err(Error::Ambiguous)
        };


        let captures = SINGLE_REGEXES[index]
            .captures(line.as_str())
            .expect("Log line matches REGEX_SET but fails SINGLE_REGEXES");

        match index {
            0 => {
                Ok(LogEntry::LogFileStart {
                    prefix: extract_prefix(&captures),
                    file: extract_into(&captures, "file"),
                    game: extract_into(&captures, "game"),
                    version: extract_parse(&captures, "version"),
                })
            },
            1 => {
                Ok(LogEntry::LogFileClosed {
                    prefix: extract_prefix(&captures),
                })
            },
            // TODO for cvar dump, process by recursion. If a non cvar_dump is found return that, otherwise return the completed cvar_dump when it has completed. (tail recursion!)
            _ => {
                panic!("Matched a unimplemented regex (index={}). The code should probably be updated", index);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::{BufReader, prelude::*};

    use crate::csgo::logs::{LogProcessor, LogEntry};

    type LogLine = String;

    #[async_trait]
    impl super::LogEntryReader<LogLine> for LogLine {
        // replace Err type with the never type once stabilized, https://github.com/rust-lang/rust/issues/35121
        async fn read_log_line(self) -> Result<String, String> {
            Ok(self.clone())
        }
    }

    async fn parse_line(line: &str) -> LogEntry {
        let logline: LogLine = line.to_string();
        let processor = LogProcessor::new(logline);
        let result = processor.read_entry().await;

        result.unwrap()
    }

    #[actix_rt::test]
    async fn log_start() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: Log file started (file "logs/L000_000_000_000_0_202001020304_000.log") (game "/home/steam/csgo/csgo") (version "7713")"#).await;
        if let super::LogEntry::LogFileStart { prefix, file, game, version } = logentry {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);

            assert_eq!(file, "logs/L000_000_000_000_0_202001020304_000.log");
            assert_eq!(game, "/home/steam/csgo/csgo");
            assert_eq!(version, 7713);
        } else {
            assert!(false)
        }
    }

    #[actix_rt::test]
    async fn log_closed() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: Log file closed"#).await;
        if let super::LogEntry::LogFileClosed { prefix } = logentry {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
        } else {
            assert!(false)
        }
    }

    #[test]
    #[ignore]
    /// Process a pile of CS:GO log files, and check if all lines can be matched.
    /// Ignored by default because not all developers have access to logs to test on.
    fn parse_log_files() {
        // TODO player regex fails on players with a "<" character in their name

        let files: Vec<String> = vec![];
        for file in files {
            let file = File::open(file).unwrap();
            let reader = BufReader::new(file);

            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        let m = super::REGEX.matches(&line);
                        if !m.matched_any() {
                            println!("{}", line);
                        }
                    }
                    Err(err) => {
                        panic!(err)
                    }
                }
            }
        }
    }
}
