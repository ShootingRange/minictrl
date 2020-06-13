use std::fmt::Debug;
use std::marker::PhantomData;
use std::str::FromStr;
use std::time::Duration;

use regex::{Captures, Match, Regex, RegexSet};

// NOTES ON LOG PROCESSING
//
// The log can be processed line by line, meaning it's possible to start processing a log file
// starting from any line. This does have one assumption, players should not be able to cause a
// line change. It's currently untested whether a player can insert a newline in there Steam
// nickname, or in a chat message, but if they can it's likely to make it's way into the log files
// because Valve doesn't insert any escaping in the log format.
//
// Two groups of log lines needs to be parsed statefully. The CVARs dump and ACCOLADE lines needs
// the context of previous lines to provide full information. If processing starts in the middle
// of one of the two, these lines can be discarded without breaking anything. The stateful
// interpretation of these can be moved outside the parser.
//
// A couple of regexes are known to fail if a player's Steam nickname contains a less than
// character, `<`. This is unfixable since some log lines contains user input in two places
// (nickname and chat message for instance), and Valve doesn't provide any escape characters.

#[derive(Debug, PartialEq)]
pub struct LogPrefix {
    pub month: i32,
    pub day: i32,
    pub year: i32,
    pub hour: i32,
    pub minute: i32,
    pub second: i32,
}

#[derive(Debug, PartialEq)]
pub enum TeamAll {
    TERRORIST,
    CT,
    UNASSIGNED,
    SPECTATOR,
    CONSOLE,
}

#[derive(Debug, PartialEq)]
pub enum Team {
    TERRORIST,
    CT,
}

#[derive(Debug, PartialEq)]
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
#[derive(Debug, PartialEq)]
pub struct Player {
    pub nick: String,
    pub entity_index: i32,
    pub id: PlayerID,
    pub team: TeamAll,
}

#[derive(Debug, PartialEq)]
pub struct Vector3 {
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Debug, PartialEq)]
pub struct KillAttributes {
    headshot: bool,
    penetrated: bool,
}

#[derive(Debug, PartialEq)]
pub enum HitGroup {
    Chest,
    Generic,
    Head,
    LeftArm,
    LeftLeg,
    Neck,
    RightArm,
    RightLeg,
    Stomach,
}

#[derive(Debug)]
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
        event: String,
    },
    /// Team triggered game event, and the event contains information about team scores.
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
        command: String,
    },
    /// Bad password during RCON authentication.
    RconBadPassword {
        prefix: LogPrefix,
        client_address: String,
    },
    /// Player switched from on team/side to another.
    SwitchedTeam {
        prefix: LogPrefix,
        /// The `team` fields defaults to `TeamAll::UNASSIGNED`, use `from` and `to` instead
        player: Player,
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
    // TODO description, see also regex
    TeamPlaying {
        prefix: LogPrefix,
        team: Team,
        // TODO should this be a enum? values: "", "NOT READY", "READY"
        readiness: Option<String>,
        /// Team name
        name: String,
    },
    /// Freeze period started.
    StartingFreezePeriod {
        prefix: LogPrefix,
    },
    /// Player left buyzone, and can no longer buy equipment until next round.
    PlayerLeftBuyzone {
        prefix: LogPrefix,
        player: Player,
        instruments: Vec<String>,
    },
    /// Player send message in team chat.
    TeamChat {
        prefix: LogPrefix,
        // Player that send the chat message
        player: Player,
        msg: String,
    },
    /// Player's money changed.
    /// Can be caused by round change, or purchase. It's not known what the "tracked" attribute indicates. The resulting money may be capped by `mp_maxmoney`.
    MoneyChanged {
        prefix: LogPrefix,
        player: Player,
        previously: i32,
        // TODO enum, values: "INCREMENT", "DECREMENT"
        operation: String,
        change: i32,
        new_amount: i32,
        instrument: Option<String>,
        tracked: bool,
    },
    /// Player purchased instrument/equipment.
    PlayerPurchased {
        prefix: LogPrefix,
        player: Player,
        instrument: String,
    },
    /// Player threw flashbang.
    ThrewFlashbang {
        prefix: LogPrefix,
        player: Player,
        location: Vector3,
        /// Flashbang entity index
        entindex: i32,
    },
    /// Player was blinded by flashbang thrown by another player
    BlindedPlayer {
        prefix: LogPrefix,
        offender: Player,
        duration: Duration,
        victim: Player,
        /// Flashbang entity index
        entindex: i32,
    },
    /// Player send message in global chat. Both teams will see these messages.
    GlobalChat {
        prefix: LogPrefix,
        player: Player,
        msg: String,
    },
    /// Player killed entity
    PlayerKilledEntity {
        prefix: LogPrefix,
        player: Player,
        location: Vector3,
        entity_name: String,
        entindex: i32,
        entity_location: Vector3,
        instrument: String,
        kill_attributes: KillAttributes,
    },
    /// Player killed another player with instrument
    PlayerKilledPlayer {
        prefix: LogPrefix,
        offender: Player,
        offender_location: Vector3,
        victim: Player,
        victim_location: Vector3,
        instrument: String,
        kill_attributes: KillAttributes,
    },
    /// Player threw smokegrenade
    PlayerThrewSmokegrenade {
        prefix: LogPrefix,
        player: Player,
        location: Vector3,
    },
    /// Player threw high explosive grenade.
    PlayerThrewHEGrenade {
        prefix: LogPrefix,
        player: Player,
        location: Vector3,
    },
    /// Player attacked another player
    PlayerAttackedPlayer {
        prefix: LogPrefix,
        offender: Player,
        offender_location: Vector3,
        victim: Player,
        victim_location: Vector3,
        instrument: String,
        damage: i32,
        damage_armor: i32,
        health: i32,
        armor: i32,
        hitgroup: HitGroup,
    },
    /// Player disconnected from game server.
    PlayerDisconnected {
        prefix: LogPrefix,
        player: Player,
        reason: String,
    },
    /// Player assisted another player in killing a third player.
    PlayerAssistedKillingPlayer {
        prefix: LogPrefix,
        offender: Player,
        victim: Player,
    },
    /// Player assisted another player in killing a third player by blinding them (flash-assisted killing).
    PlayerAssistedBlindingPlayer {
        prefix: LogPrefix,
        offender: Player,
        victim: Player,
    },
    /// Molotov projectile spawned
    SpawnedMolotov {
        prefix: LogPrefix,
        location_x: f32,
        location_y: f32,
        location_z: f32,
        velocity_x: f32,
        velocity_y: f32,
        velocity_z: f32,
    },
    /// Player threw molotov
    ThrewMolotov {
        prefix: LogPrefix,
        player: Player,
        location: Vector3,
    },
    /// Player connceted to game server.
    PlayerConnected {
        prefix: LogPrefix,
        player: Player,
        address: String,
    },
    /// SteamID of player was validated.
    ValidatedSteamID {
        prefix: LogPrefix,
        player: Player,
    },
    /// Team ended match with given score, and number of player participating.
    TeamScored {
        prefix: LogPrefix,
        team: Team,
        score: i32,
        player_count: i32,
    },
    /// Player threw decoy.
    ThrewDecoy {
        prefix: LogPrefix,
        player: Player,
        location: Vector3,
    },
    /// Match resumed.
    MatchResumed {
        prefix: LogPrefix,
    },
    /// Match paused.
    MatchPaused {
        prefix: LogPrefix,
    },
    /// Player was killed by bomb
    KilledByBomb {
        prefix: LogPrefix,
        player: Player,
        location: Vector3,
    },
    // TODO description
    Accolade {
        prefix: LogPrefix,
        categorie: String,
        player: String,
        player_entindex: i32,
        value: f32,
        pos: i32,
        score: f32,
    },
    /// Game ended.
    /// CT and T score might be swapped, this needs validation.
    GameOver {
        prefix: LogPrefix,
        mode: String,
        map_group: String,
        map: String,
        ct_score: i32,
        t_score: i32,
        time: Duration,
    },
    /// Player changed their nickname.
    ChangedNickname {
        prefix: LogPrefix,
        player: Player,
        new_nickname: String,
    },
    /// Player committed suicide with instrument
    CommittedSuicide {
        prefix: LogPrefix,
        player: Player,
        location: Vector3,
        instrument: String,
    },
    // TODO description
    ServerMessage {
        prefix: LogPrefix,
        message: String,
    },
    /// Failed to validate user authentication ticket.
    /// Error codes are described in the [`steam_api.h` documentation](https://partner.steamgames.com/doc/api/steam_api#EAuthSessionResponse).
    SteamAuthFailure {
        prefix: LogPrefix,
        nickname: String,
        failure_code: i32, // Should this be a enum?
    },
    /// META mod has loaded plugins.
    MetaModPluginsLoaded {
        prefix: LogPrefix,
        /// Number of plugins loaded.
        loaded: i32,
        /// Number of plugins already loaded.
        ///
        /// 0 if the log entry doesn't have `(1 already loaded)`
        preloaded: i32,
    },
}

lazy_static! {
    /// Prefix for a CS:GO log line
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
        LOG_PREFIX.clone() + r#"rcon from "(?P<client_address>\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}:\d{1,5})": command "(?P<command>.*)"$"#,
        LOG_PREFIX.clone() + r#"rcon from "(?P<client_address>\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}:\d{1,5})": Bad Password$"#,
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT))>" switched from team <(?P<from_side>(Unassigned|TERRORIST|CT|Spectator)?)> to <(?P<to_side>(Unassigned|TERRORIST|CT|Spectator)?)>$"#,
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" picked up "(?P<instrument>[^"]*)"$"#,
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" dropped "(?P<instrument>[^"]*)"$"#,
        LOG_PREFIX.clone() + r#"Team playing "(?P<side>(CT|TERRORIST))": (\[(?P<readiness>(NOT )?READY)\] )?(?P<team>.*)$"#,
        LOG_PREFIX.clone() + r#"Starting Freeze period$"#,
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" left buyzone with \[ (?P<instruments>([A-Za-z0-9_]*(\(\d+\))? )*)\]$"#,
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" say_team "(?P<msg>.*)"$"#,
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" money change (?P<money_prev>\d+)(?P<money_op>[\+-])(?P<money_diff>\d+) = \$(?P<money_after>\d+)( \((?P<tracked>tracked)\)( \(purchase: (?P<instrument>[A-Za-z0-9_]*(\(\d+\))?)\))?)?$"#,
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" purchased "(?P<instrument>[A-Za-z0-9_]*(\(\d+\))?)"$"#,
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" threw flashbang \[(?P<loc_x>-?\d+) (?P<loc_y>-?\d+) (?P<loc_z>-?\d+)\] flashbang entindex (?P<entindex>\d+)\)$"#, // Extra bracket at end
        LOG_PREFIX.clone() + r#""(?P<offender_nick>[^<]*)<(?P<offender_entindex>\d+)><(?P<offender_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<offender_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" blinded for (?P<duration_sec>\d+)\.(?P<duration_decimal>\d{2}) by "(?P<victim_nick>[^<]*)<(?P<victim_entindex>\d+)><(?P<victim_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<victim_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" from flashbang entindex (?P<entindex>\d+) $"#, // Trailing space
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" say "(?P<msg>.*)"$"#, // Message can contain double quotes
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" \[(?P<loc_x>-?\d+) (?P<loc_y>-?\d+) (?P<loc_z>-?\d+)\] killed other "(?P<ent>[^<]*)<(?P<entindex>\d+)>" \[(?P<ent_x>-?\d+) (?P<ent_y>-?\d+) (?P<ent_z>-?\d+)\] with "(?P<instrument>[A-Za-z0-9_]*(\(\d+\))?)"( \((?P<kill_attributes>(headshot|penetrated|headshot penetrated))\))?$"#,
        LOG_PREFIX.clone() + r#""(?P<offender_nick>[^<]*)<(?P<offender_entindex>\d+)><(?P<offender_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<offender_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" \[(?P<loc_x>-?\d+) (?P<loc_y>-?\d+) (?P<loc_z>-?\d+)\] killed "(?P<victim_nick>[^<]*)<(?P<victim_entindex>\d+)><(?P<victim_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<victim_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" \[(?P<victim_x>-?\d+) (?P<victim_y>-?\d+) (?P<victim_z>-?\d+)\] with "(?P<instrument>[A-Za-z0-9_]*(\(\d+\))?)"( \((?P<kill_attributes>(headshot|penetrated|headshot penetrated))\))?$"#,
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" threw smokegrenade \[(?P<loc_x>-?\d+) (?P<loc_y>-?\d+) (?P<loc_z>-?\d+)\]$"#,
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" threw hegrenade \[(?P<loc_x>-?\d+) (?P<loc_y>-?\d+) (?P<loc_z>-?\d+)\]$"#,
        LOG_PREFIX.clone() + r#""(?P<offender_nick>[^<]*)<(?P<offender_entindex>\d+)><(?P<offender_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<offender_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" \[(?P<offender_x>-?\d+) (?P<offender_y>-?\d+) (?P<offender_z>-?\d+)\] attacked "(?P<victim_nick>[^<]*)<(?P<victim_entindex>\d+)><(?P<victim_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<victim_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" \[(?P<victim_x>-?\d+) (?P<victim_y>-?\d+) (?P<victim_z>-?\d+)\] with "(?P<instrument>[A-Za-z0-9_]*(\(\d+\))?)" \(damage "(?P<damage>\d+)"\) \(damage_armor "(?P<damage_armor>\d+)"\) \(health "(?P<health>\d+)"\) \(armor "(?P<armor>\d+)"\) \(hitgroup "(?P<hitgroup>(chest|generic|head|left arm|left leg|neck|right arm|right leg|stomach))"\)$"#,
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" disconnected \(reason "(?P<reason>[^"]*)"\)$"#,
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" assisted killing "(?P<player_killed_nick>[^<]*)<(?P<player_killed_entindex>\d+)><(?P<player_killed_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_killed_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>"$"#,
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" flash-assisted killing "(?P<player_killed_nick>[^<]*)<(?P<player_killed_entindex>\d+)><(?P<player_killed_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_killed_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>"$"#,
        LOG_PREFIX.clone() + r#"Molotov projectile spawned at (?P<loc_x>-?\d+\.\d+) (?P<loc_y>-?\d+\.\d+) (?P<loc_z>-?\d+\.\d+), velocity (?P<vec_x>-?\d+\.\d+) (?P<vec_y>-?\d+\.\d+) (?P<vec_z>-?\d+\.\d+)$"#,
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" threw molotov \[(?P<loc_x>-?\d+) (?P<loc_y>-?\d+) (?P<loc_z>-?\d+)\]$"#,
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" connected, address "(?P<ip_address>[^"]*)"$"#,
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" STEAM USERID validated$"#,
        LOG_PREFIX.clone() + r#"Team "(?P<side>(CT|TERRORIST))" scored "(?P<score>\d+)" with "(?P<player_count>\d+)" players$"#,
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" threw decoy \[(?P<loc_x>-?\d+) (?P<loc_y>-?\d+) (?P<loc_z>-?\d+)\]$"#,
        LOG_PREFIX.clone() + r#"Match pause is disabled - mp_unpause_match$"#,
        LOG_PREFIX.clone() + r#"Match pause is enabled - mp_pause_match$"#,
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" \[(?P<loc_x>-?\d+) (?P<loc_y>-?\d+) (?P<loc_z>-?\d+)\] was killed by the bomb\.$"#,
        LOG_PREFIX.clone() + r#"ACCOLADE, FINAL: \{(?P<categorie>[^\}]*)\},\s+(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)>,\s+VALUE: (?P<value>\d+\.\d+),\s+POS: (?P<pos>\d+),\s+SCORE: (?P<score>\d+\.\d+)$"#,
        LOG_PREFIX.clone() + r#"Game Over: (?P<mode>[A-Za-z0-9_]+) (?P<map_group>[A-Za-z0-9_]+) (?P<map>[A-Za-z0-9_]+) score (?P<ct_score>\d+):(?P<t_score>\d+) after (?P<time>\d+) min$"#,
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" changed name to "(?P<new_nick>[^"]*)"$"#,
        LOG_PREFIX.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" \[(?P<loc_x>-?\d+) (?P<loc_y>-?\d+) (?P<loc_z>-?\d+)\] committed suicide with "(?P<instrument>[^"]*)"$"#,
        LOG_PREFIX.clone() + r#"server_message: "(?P<msg>[^"]*)"$"#,
        LOG_PREFIX.clone() + r#"STEAMAUTH: Client (?P<player_nick>.*) received failure code (?P<code>\d+)$"#,
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
where
    E::Err: Debug,
{
    // TODO build expect messages at compile time using something like the concat! macro. A combination of macros and functions could be effective, https://godbolt.org/z/bAJUG9
    captures
        .name(group)
        .expect(format!("no match for capture \"{}\"", group).as_str())
        .as_str()
        .parse()
        .unwrap()
    //.expect(format!("Failed to parse \"{}\"", group).as_str())
}

fn extract_parse_optional<E: FromStr>(captures: &Captures, group: &str) -> Option<E>
where
    E::Err: Debug,
{
    match captures.name(group) {
        None => None,
        Some(group) => {
            // TODO build expect messages at compile time using something like the concat! macro. A combination of macros and functions could be effective, https://godbolt.org/z/bAJUG9
            Some(group.as_str().parse().unwrap())
            //.expect(format!("Failed to parse \"{}\"", group).as_str())
        }
    }
}

fn extract_into<'t, E: From<&'t str>>(captures: &Captures<'t>, group: &str) -> E {
    captures
        .name(group)
        .expect(format!("no match for capture \"{}\"", group).as_str())
        .as_str()
        .into()
}

fn extract_optional_into<'t, E: From<&'t str>>(captures: &Captures<'t>, group: &str) -> Option<E> {
    match captures.name(group) {
        None => None,
        Some(group) => Some(group.as_str().into()),
    }
}

fn extract_str<'t>(captures: &Captures<'t>, group: &str) -> &'t str {
    captures
        .name(group)
        .expect(format!("no match for capture \"{}\"", group).as_str())
        .as_str()
}

fn extract_optional_str<'t>(captures: &Captures<'t>, group: &str) -> Option<&'t str> {
    match captures.name(group) {
        None => None,
        Some(group) => Some(group.as_str()),
    }
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

fn extract_team(captures: &Captures, group: &str) -> Team {
    let team = captures
        .name(group)
        .expect(format!("no match for capture \"{}\"", group).as_str())
        .as_str();

    match team {
        "TERRORIST" => Team::TERRORIST,
        "CT" => Team::CT,
        _ => panic!(format!("Unexpected Team type ({})", team)),
    }
}

fn extract_team_all(captures: &Captures, group: &str) -> TeamAll {
    let team = captures
        .name(group)
        .expect(format!("no match for capture \"{}\"", group).as_str())
        .as_str();

    match team {
        "Unassigned" => TeamAll::UNASSIGNED,
        "" => TeamAll::UNASSIGNED,
        "TERRORIST" => TeamAll::TERRORIST,
        "CT" => TeamAll::CT,
        "Spectator" => TeamAll::SPECTATOR,
        "Console" => TeamAll::CONSOLE,
        _ => panic!(format!("Unexpected TeamAll type ({})", team)),
    }
}

fn extract_player_id(captures: &Captures, group: &str) -> PlayerID {
    let team = captures
        .name(group)
        .expect(format!("no match for capture \"{}\"", group).as_str())
        .as_str();

    // This is not exactly equivalent to the regex,
    // but that shouldn't matter since we won't get anything that doesn't match the regex
    // and the prefix is unambiguous.
    if team.starts_with("STEAM_") {
        return PlayerID::STAMID(team.to_string());
    }

    match team {
        "BOT" => PlayerID::BOT,
        "Console" => PlayerID::CONSOLE,
        _ => panic!(format!("Unexpected PlayerID type ({})", team)),
    }
}

// TODO convert this to a macro such that we don't have to do string building
fn extract_player(captures: &Captures, prefix: &str) -> Player {
    Player {
        nick: extract_into(&captures, (prefix.to_owned() + "_nick").as_str()),
        entity_index: extract_parse(&captures, (prefix.to_owned() + "_entindex").as_str()),
        id: extract_player_id(&captures, (prefix.to_owned() + "_id").as_str()),
        team: extract_team_all(&captures, (prefix.to_owned() + "_team").as_str()),
    }
}

fn extract_vector3(captures: &Captures, prefix: &str) -> Vector3 {
    Vector3 {
        x: extract_parse(&captures, (prefix.to_owned() + "_x").as_str()),
        y: extract_parse(&captures, (prefix.to_owned() + "_y").as_str()),
        z: extract_parse(&captures, (prefix.to_owned() + "_z").as_str()),
    }
}

fn extract_kill_attributes(captures: &Captures) -> KillAttributes {
    let capture = captures.name("kill_attributes");

    if let Option::None = capture {
        return KillAttributes {
            headshot: false,
            penetrated: false,
        };
    }

    let attributes = capture
        .expect("no match for capture \"attributes\"")
        .as_str();

    match attributes {
        "headshot" => KillAttributes {
            headshot: true,
            penetrated: false,
        },
        "penetrated" => KillAttributes {
            headshot: false,
            penetrated: true,
        },
        "headshot penetrated" => KillAttributes {
            headshot: true,
            penetrated: true,
        },
        _ => panic!(format!("Unexpected HitGroup type ({})", attributes)),
    }
}

fn extract_hitgroup(captures: &Captures) -> HitGroup {
    let hitgroup = captures
        .name("hitgroup")
        .expect("no match for capture \"hitgroup\"")
        .as_str();

    match hitgroup {
        "chest" => HitGroup::Chest,
        "generic" => HitGroup::Generic,
        "head" => HitGroup::Head,
        "left arm" => HitGroup::LeftArm,
        "left leg" => HitGroup::LeftLeg,
        "neck" => HitGroup::Neck,
        "right arm" => HitGroup::RightArm,
        "right leg" => HitGroup::RightLeg,
        "stomach" => HitGroup::Stomach,
        _ => panic!(format!("Unexpected HitGroup type ({})", hitgroup)),
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

        let matchs: Vec<usize> = REGEX.matches(&line).iter().collect();

        let index = match matchs.len() {
            // No regex matched the log line
            0 => return Result::Err(Error::Unknown(line)),
            // Success
            1 => matchs[0],
            // If more than one regex matches the log line can't parsed decidedly
            // ASSUMPTION: usize can't represent a number less than zero
            _ => return Result::Err(Error::Ambiguous),
        };

        let captures = SINGLE_REGEXES[index]
            .captures(line.as_str())
            .expect("Log line matches REGEX_SET but fails SINGLE_REGEXES");

        match index {
            0 => Ok(LogEntry::LogFileStart {
                prefix: extract_prefix(&captures),
                file: extract_into(&captures, "file"),
                game: extract_into(&captures, "game"),
                version: extract_parse(&captures, "version"),
            }),
            1 => Ok(LogEntry::LogFileClosed {
                prefix: extract_prefix(&captures),
            }),
            2 => Ok(LogEntry::WorldTriggeredEvent {
                prefix: extract_prefix(&captures),
                event: extract_into(&captures, "event"),
            }),
            3 => Ok(LogEntry::WorldTriggeredEventMap {
                prefix: extract_prefix(&captures),
                event: extract_into(&captures, "event"),
                map: extract_into(&captures, "map"),
            }),
            4 => Ok(LogEntry::WorldTriggeredEventScore {
                prefix: extract_prefix(&captures),
                event: extract_into(&captures, "event"),
                ct_score: extract_parse(&captures, "ct"),
                t_score: extract_parse(&captures, "t"),
            }),
            5 => Ok(LogEntry::PlayerTriggeredEvent {
                prefix: extract_prefix(&captures),
                player: extract_player(&captures, "player"),
                event: extract_into(&captures, "event"),
            }),
            6 => Ok(LogEntry::TeamTriggeredEventScore {
                prefix: extract_prefix(&captures),
                team: extract_team(&captures, "team"),
                event: extract_into(&captures, "event"),
                ct_score: extract_parse(&captures, "ct"),
                t_score: extract_parse(&captures, "t"),
            }),
            7 => Ok(LogEntry::LoadingMap {
                prefix: extract_prefix(&captures),
                map: extract_into(&captures, "map"),
            }),
            // TODO for cvar dump, process by recursion. If a non cvar_dump is found return that, otherwise return the completed cvar_dump when it has completed. (tail recursion!)
            8 => {
                // begin cvar dump
                // TODO write stateful handling of LogEntry::CvarDump
                unimplemented!()
            }
            9 => {
                // cvar from dump
                unimplemented!()
            }
            10 => {
                // ended cvar dump
                unimplemented!()
            }
            11 => Ok(LogEntry::StartedMap {
                prefix: extract_prefix(&captures),
                map: extract_into(&captures, "map"),
                crc: extract_into(&captures, "crc"),
            }),
            12 => Ok(LogEntry::Cvar {
                prefix: extract_prefix(&captures),
                key: extract_into(&captures, "cvar_key"),
                value: extract_into(&captures, "cvar_value"),
            }),
            13 => Ok(LogEntry::PlayerEnteredGame {
                prefix: extract_prefix(&captures),
                player: extract_player(&captures, "player"),
            }),
            14 => Ok(LogEntry::Get5Event {
                prefix: extract_prefix(&captures),
                json: extract_into(&captures, "json"),
            }),
            15 => Ok(LogEntry::RconCommand {
                prefix: extract_prefix(&captures),
                client_address: extract_into(&captures, "client_address"),
                command: extract_into(&captures, "command"),
            }),
            16 => Ok(LogEntry::RconBadPassword {
                prefix: extract_prefix(&captures),
                client_address: extract_into(&captures, "client_address"),
            }),
            17 => Ok(LogEntry::SwitchedTeam {
                prefix: extract_prefix(&captures),
                player: Player {
                    nick: extract_into(&captures, "player_nick"),
                    entity_index: extract_parse(&captures, "player_entindex"),
                    id: extract_player_id(&captures, "player_id"),
                    team: TeamAll::UNASSIGNED,
                },
                from: extract_team_all(&captures, "from_side"),
                to: extract_team_all(&captures, "to_side"),
            }),
            18 => Ok(LogEntry::PlayerPickedUp {
                prefix: extract_prefix(&captures),
                player: extract_player(&captures, "player"),
                instrument: extract_into(&captures, "instrument"),
            }),
            19 => Ok(LogEntry::PlayerDropped {
                prefix: extract_prefix(&captures),
                player: extract_player(&captures, "player"),
                instrument: extract_into(&captures, "instrument"),
            }),
            20 => Ok(LogEntry::TeamPlaying {
                prefix: extract_prefix(&captures),
                team: extract_team(&captures, "side"),
                readiness: extract_optional_into(&captures, "readiness"),
                name: extract_into(&captures, "team"),
            }),
            21 => Ok(LogEntry::StartingFreezePeriod {
                prefix: extract_prefix(&captures),
            }),
            22 => {
                // An empty equipment list looks like this `[ ]`
                // The regex doesn't capture the first space,
                // therefor there is no superfluous element generated at the beginning of the list.
                // When the list of instruments is non-empty, a trailing space has to be removed,
                // otherwise a trailing empty element is produced.
                let raw_instruments = extract_into::<String>(&captures, "instruments");
                let raw_instruments = raw_instruments.trim_end_matches(" ");
                let instruments = raw_instruments
                    .split(" ")
                    .map(|instrument| instrument.to_string())
                    .collect::<Vec<String>>();

                Ok(LogEntry::PlayerLeftBuyzone {
                    prefix: extract_prefix(&captures),
                    player: extract_player(&captures, "player"),
                    instruments,
                })
            }
            23 => Ok(LogEntry::TeamChat {
                prefix: extract_prefix(&captures),
                player: extract_player(&captures, "player"),
                msg: extract_into(&captures, "msg"),
            }),
            24 => Ok(LogEntry::MoneyChanged {
                prefix: extract_prefix(&captures),
                player: extract_player(&captures, "player"),
                previously: extract_parse(&captures, "money_prev"),
                operation: extract_into(&captures, "money_op"),
                change: extract_parse(&captures, "money_diff"),
                new_amount: extract_parse(&captures, "money_after"),
                instrument: extract_optional_into(&captures, "instrument"),
                tracked: captures.name("tracked").is_some(),
            }),
            25 => Ok(LogEntry::PlayerPurchased {
                prefix: extract_prefix(&captures),
                player: extract_player(&captures, "player"),
                instrument: extract_into(&captures, "instrument"),
            }),
            26 => Ok(LogEntry::ThrewFlashbang {
                prefix: extract_prefix(&captures),
                player: extract_player(&captures, "player"),
                location: extract_vector3(&captures, "loc"),
                entindex: extract_parse(&captures, "entindex"),
            }),
            27 => Ok(LogEntry::BlindedPlayer {
                prefix: extract_prefix(&captures),
                offender: extract_player(&captures, "offender"),
                duration: Duration::from_secs(extract_parse::<u64>(&captures, "duration_sec"))
                    + Duration::from_millis(
                        extract_parse::<u64>(&captures, "duration_decimal") * 10,
                    ),
                victim: extract_player(&captures, "victim"),
                entindex: extract_parse(&captures, "entindex"),
            }),
            28 => Ok(LogEntry::GlobalChat {
                prefix: extract_prefix(&captures),
                player: extract_player(&captures, "player"),
                msg: extract_into(&captures, "msg"),
            }),
            29 => Ok(LogEntry::PlayerKilledEntity {
                prefix: extract_prefix(&captures),
                player: extract_player(&captures, "player"),
                location: extract_vector3(&captures, "loc"),
                entity_name: extract_into(&captures, "ent"),
                entindex: extract_parse(&captures, "entindex"),
                entity_location: extract_vector3(&captures, "ent"),
                instrument: extract_into(&captures, "instrument"),
                kill_attributes: extract_kill_attributes(&captures),
            }),
            30 => Ok(LogEntry::PlayerKilledPlayer {
                prefix: extract_prefix(&captures),
                offender: extract_player(&captures, "offender"),
                offender_location: extract_vector3(&captures, "loc"),
                victim: extract_player(&captures, "victim"),
                victim_location: extract_vector3(&captures, "victim"),
                instrument: extract_into(&captures, "instrument"),
                kill_attributes: extract_kill_attributes(&captures),
            }),
            31 => Ok(LogEntry::PlayerThrewSmokegrenade {
                prefix: extract_prefix(&captures),
                player: extract_player(&captures, "player"),
                location: extract_vector3(&captures, "loc"),
            }),
            32 => Ok(LogEntry::PlayerThrewHEGrenade {
                prefix: extract_prefix(&captures),
                player: extract_player(&captures, "player"),
                location: extract_vector3(&captures, "loc"),
            }),
            33 => Ok(LogEntry::PlayerAttackedPlayer {
                prefix: extract_prefix(&captures),
                offender: extract_player(&captures, "offender"),
                offender_location: extract_vector3(&captures, "offender"),
                victim: extract_player(&captures, "victim"),
                victim_location: extract_vector3(&captures, "victim"),
                instrument: extract_into(&captures, "instrument"),
                damage: extract_parse(&captures, "damage"),
                damage_armor: extract_parse(&captures, "damage_armor"),
                health: extract_parse(&captures, "health"),
                armor: extract_parse(&captures, "armor"),
                hitgroup: extract_hitgroup(&captures),
            }),
            34 => Ok(LogEntry::PlayerDisconnected {
                prefix: extract_prefix(&captures),
                player: extract_player(&captures, "player"),
                reason: extract_into(&captures, "reason"),
            }),
            35 => Ok(LogEntry::PlayerAssistedKillingPlayer {
                prefix: extract_prefix(&captures),
                offender: extract_player(&captures, "player"),
                victim: extract_player(&captures, "player_killed"),
            }),
            36 => Ok(LogEntry::PlayerAssistedBlindingPlayer {
                prefix: extract_prefix(&captures),
                offender: extract_player(&captures, "player"),
                victim: extract_player(&captures, "player_killed"),
            }),
            37 => Ok(LogEntry::SpawnedMolotov {
                prefix: extract_prefix(&captures),
                location_x: extract_parse(&captures, "loc_x"),
                location_y: extract_parse(&captures, "loc_y"),
                location_z: extract_parse(&captures, "loc_z"),
                velocity_x: extract_parse(&captures, "vec_x"),
                velocity_y: extract_parse(&captures, "vec_y"),
                velocity_z: extract_parse(&captures, "vec_z"),
            }),
            38 => Ok(LogEntry::ThrewMolotov {
                prefix: extract_prefix(&captures),
                player: extract_player(&captures, "player"),
                location: extract_vector3(&captures, "loc"),
            }),
            39 => Ok(LogEntry::PlayerConnected {
                prefix: extract_prefix(&captures),
                player: extract_player(&captures, "player"),
                address: extract_into(&captures, "ip_address"),
            }),
            40 => Ok(LogEntry::ValidatedSteamID {
                prefix: extract_prefix(&captures),
                player: extract_player(&captures, "player"),
            }),
            41 => Ok(LogEntry::TeamScored {
                prefix: extract_prefix(&captures),
                team: extract_team(&captures, "side"),
                score: extract_parse(&captures, "score"),
                player_count: extract_parse(&captures, "player_count"),
            }),
            42 => Ok(LogEntry::ThrewDecoy {
                prefix: extract_prefix(&captures),
                player: extract_player(&captures, "player"),
                location: extract_vector3(&captures, "loc"),
            }),
            43 => Ok(LogEntry::MatchResumed {
                prefix: extract_prefix(&captures),
            }),
            44 => Ok(LogEntry::MatchPaused {
                prefix: extract_prefix(&captures),
            }),
            45 => Ok(LogEntry::KilledByBomb {
                prefix: extract_prefix(&captures),
                player: extract_player(&captures, "player"),
                location: extract_vector3(&captures, "loc"),
            }),
            46 => Ok(LogEntry::Accolade {
                prefix: extract_prefix(&captures),
                categorie: extract_into(&captures, "categorie"),
                player: extract_into(&captures, "player_nick"),
                player_entindex: extract_parse(&captures, "player_entindex"),
                value: extract_parse(&captures, "value"),
                pos: extract_parse(&captures, "pos"),
                score: extract_parse(&captures, "score"),
            }),
            47 => {
                let x = extract_parse::<u64>(&captures, "time");
                Ok(LogEntry::GameOver {
                    prefix: extract_prefix(&captures),
                    mode: extract_into(&captures, "mode"),
                    map_group: extract_into(&captures, "map_group"),
                    map: extract_into(&captures, "map"),
                    ct_score: extract_parse(&captures, "ct_score"),
                    t_score: extract_parse(&captures, "t_score"),
                    time: Duration::from_secs(x * 60),
                })
            }
            48 => Ok(LogEntry::ChangedNickname {
                prefix: extract_prefix(&captures),
                player: extract_player(&captures, "player"),
                new_nickname: extract_into(&captures, "new_nick"),
            }),
            49 => Ok(LogEntry::CommittedSuicide {
                prefix: extract_prefix(&captures),
                player: extract_player(&captures, "player"),
                location: extract_vector3(&captures, "loc"),
                instrument: extract_into(&captures, "instrument"),
            }),
            50 => Ok(LogEntry::ServerMessage {
                prefix: extract_prefix(&captures),
                message: extract_into(&captures, "msg"),
            }),
            51 => Ok(LogEntry::SteamAuthFailure {
                prefix: extract_prefix(&captures),
                nickname: extract_into(&captures, "player_nick"),
                failure_code: extract_parse(&captures, "code"),
            }),
            52 => Ok(LogEntry::MetaModPluginsLoaded {
                prefix: extract_prefix(&captures),
                loaded: extract_parse(&captures, "plugins_loaded"),
                preloaded: extract_parse_optional(&captures, "plugins_preloaded")
                    .unwrap_or_default(),
            }),
            _ => {
                panic!(
                    "Matched a unimplemented regex (index={}). The code should probably be updated",
                    index
                );
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::{prelude::*, BufReader};

    use actix::clock::Duration;

    use crate::csgo::logs::HitGroup::Head;
    use crate::csgo::logs::{
        HitGroup, KillAttributes, LogEntry, LogProcessor, Player, Team, TeamAll,
    };

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
        if let super::LogEntry::LogFileStart {
            prefix,
            file,
            game,
            version,
        } = logentry
        {
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
            panic!("wrong LogEntry type received, {:#?}", logentry)
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
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_world_triggered_event() {
        let logentry =
            parse_line(r#"L 01/02/2020 - 03:04:05: World triggered "Round_Start""#).await;
        if let super::LogEntry::WorldTriggeredEvent { prefix, event } = logentry {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(event, "Round_Start")
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_world_triggered_event_map() {
        let logentry =
            parse_line(r#"L 01/02/2020 - 03:04:05: World triggered "Match_Start" on "de_inferno""#)
                .await;
        if let super::LogEntry::WorldTriggeredEventMap { prefix, event, map } = logentry {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(event, "Match_Start");
            assert_eq!(map, "de_inferno");
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_world_triggered_event_score() {
        let logentry = parse_line(
            r#"L 01/02/2020 - 03:04:05: World triggered "SFUI_Notice_Round_Draw" (CT "4") (T "0")"#,
        )
        .await;
        if let super::LogEntry::WorldTriggeredEventScore {
            prefix,
            event,
            ct_score,
            t_score,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(event, "SFUI_Notice_Round_Draw");
            assert_eq!(ct_score, 4);
            assert_eq!(t_score, 0);
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_player_triggered_event() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: "foobar<20><STEAM_1:1:12345678><CT>" triggered "Begin_Bomb_Defuse_Without_Kit""#).await;
        if let super::LogEntry::PlayerTriggeredEvent {
            prefix,
            player,
            event,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(player.nick, "foobar");
            assert_eq!(player.entity_index, 20);
            assert_eq!(
                player.id,
                super::PlayerID::STAMID("STEAM_1:1:12345678".to_string())
            );
            assert_eq!(player.team, super::TeamAll::CT);
            assert_eq!(event, "Begin_Bomb_Defuse_Without_Kit");
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_team_triggered_event_score() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: Team "TERRORIST" triggered "SFUI_Notice_Target_Bombed" (CT "0") (T "4")"#).await;
        if let super::LogEntry::TeamTriggeredEventScore {
            prefix,
            team,
            event,
            ct_score,
            t_score,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(team, super::Team::TERRORIST);
            assert_eq!(event, "SFUI_Notice_Target_Bombed");
            assert_eq!(ct_score, 0);
            assert_eq!(t_score, 4)
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_loading_map() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: Loading map "de_dust2""#).await;
        if let super::LogEntry::LoadingMap { prefix, map } = logentry {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(map, "de_dust2");
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    // TODO LogEntry::CvarDump, this needs a custom test for multiline

    #[actix_rt::test]
    async fn log_started_map() {
        let logentry =
            parse_line(r#"L 01/02/2020 - 03:04:05: Started map "de_inferno" (CRC "-1384208105")"#)
                .await;
        if let super::LogEntry::StartedMap { prefix, map, crc } = logentry {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(map, "de_inferno");
            assert_eq!(crc, "-1384208105");
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_cvar() {
        let logentry =
            parse_line(r#"L 01/02/2020 - 03:04:05: server_cvar: "mp_friendlyfire" "0""#).await;
        if let super::LogEntry::Cvar { prefix, key, value } = logentry {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(key, "mp_friendlyfire");
            assert_eq!(value, "0");
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_player_entered_game_player() {
        let logentry = parse_line(
            r#"L 01/02/2020 - 03:04:05: "foobar<20><STEAM_1:1:12345678><>" entered the game"#,
        )
        .await;
        if let super::LogEntry::PlayerEnteredGame { prefix, player } = logentry {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(player.nick, "foobar");
            assert_eq!(player.entity_index, 20);
            assert_eq!(
                player.id,
                super::PlayerID::STAMID("STEAM_1:1:12345678".to_string())
            );
            assert_eq!(player.team, super::TeamAll::UNASSIGNED);
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_player_entered_game_bot() {
        let logentry =
            parse_line(r#"L 01/02/2020 - 03:04:05: "foobar<2><BOT><>" entered the game"#).await;
        if let super::LogEntry::PlayerEnteredGame { prefix, player } = logentry {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(player.nick, "foobar");
            assert_eq!(player.entity_index, 2);
            assert_eq!(player.id, super::PlayerID::BOT);
            assert_eq!(player.team, super::TeamAll::UNASSIGNED);
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_get5_event() {
        // the "matchid" field name is supposed to have a GRAVE ACCENT character in it, this is a bug in get5
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: get5_event: {"matchid`":"","params":{"client":"none","map_number":0,"map_name":"de_dust2"},"event":"player_disconnect"}"#).await;
        if let super::LogEntry::Get5Event { prefix, json } = logentry {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(
                json,
                r#"{"matchid`":"","params":{"client":"none","map_number":0,"map_name":"de_dust2"},"event":"player_disconnect"}"#
            );
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_rcon_command() {
        let logentry = parse_line(
            r#"L 01/02/2020 - 03:04:05: rcon from "10.0.0.100:36686": command "status""#,
        )
        .await;
        if let super::LogEntry::RconCommand {
            prefix,
            client_address,
            command,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(client_address, "10.0.0.100:36686");
            assert_eq!(command, "status");
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_rcon_bad_password() {
        let logentry =
            parse_line(r#"L 01/02/2020 - 03:04:05: rcon from "10.0.0.100:49904": Bad Password"#)
                .await;
        if let super::LogEntry::RconBadPassword {
            prefix,
            client_address,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(client_address, "10.0.0.100:49904");
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_switched_team() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: "foobar<20><STEAM_1:1:12345678>" switched from team <Unassigned> to <TERRORIST>"#).await;
        if let super::LogEntry::SwitchedTeam {
            prefix,
            player,
            from,
            to,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(player.nick, "foobar");
            assert_eq!(player.entity_index, 20);
            assert_eq!(
                player.id,
                super::PlayerID::STAMID("STEAM_1:1:12345678".to_string())
            );
            //assert_eq!(player.team, ???); // Omitted in the log because it's specified in `from` and `to`
            assert_eq!(from, TeamAll::UNASSIGNED);
            assert_eq!(to, TeamAll::TERRORIST);
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_player_picked_up() {
        let logentry = parse_line(
            r#"L 01/02/2020 - 03:04:05: "foobar<20><STEAM_1:1:12345678><CT>" picked up "deagle""#,
        )
        .await;
        if let super::LogEntry::PlayerPickedUp {
            prefix,
            player,
            instrument,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(player.nick, "foobar");
            assert_eq!(player.entity_index, 20);
            assert_eq!(
                player.id,
                super::PlayerID::STAMID("STEAM_1:1:12345678".to_string())
            );
            assert_eq!(player.team, super::TeamAll::CT);
            assert_eq!(instrument, "deagle");
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_player_dropped() {
        let logentry = parse_line(
            r#"L 01/02/2020 - 03:04:05: "foobar<20><STEAM_1:1:12345678><CT>" dropped "awp""#,
        )
        .await;
        if let super::LogEntry::PlayerDropped {
            prefix,
            player,
            instrument,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(player.nick, "foobar");
            assert_eq!(player.entity_index, 20);
            assert_eq!(
                player.id,
                super::PlayerID::STAMID("STEAM_1:1:12345678".to_string())
            );
            assert_eq!(player.team, super::TeamAll::CT);
            assert_eq!(instrument, "awp");
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_team_playing_not_ready() {
        let logentry =
            parse_line(r#"L 01/02/2020 - 03:04:05: Team playing "CT": [NOT READY] heyo"#).await;
        if let super::LogEntry::TeamPlaying {
            prefix,
            team,
            readiness,
            name,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(team, Team::CT);
            assert_eq!(readiness, Some("NOT READY".to_string()));
            assert_eq!(name, "heyo");
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_team_playing_ready() {
        let logentry =
            parse_line(r#"L 01/02/2020 - 03:04:05: Team playing "TERRORIST": [READY] heyo"#).await;
        if let super::LogEntry::TeamPlaying {
            prefix,
            team,
            readiness,
            name,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(team, Team::TERRORIST);
            assert_eq!(readiness, Some("READY".to_string()));
            assert_eq!(name, "heyo");
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_team_playing_unspecified() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: Team playing "CT": heyo"#).await;
        if let super::LogEntry::TeamPlaying {
            prefix,
            team,
            readiness,
            name,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(team, Team::CT);
            assert_eq!(readiness, None);
            assert_eq!(name, "heyo");
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_starting_freeze_period() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: Starting Freeze period"#).await;
        if let super::LogEntry::StartingFreezePeriod { prefix } = logentry {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_player_left_buyzone() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: "foobar<20><STEAM_1:1:12345678><CT>" left buyzone with [ weapon_knife weapon_hkp2000 weapon_famas weapon_hegrenade kevlar(100) helmet ]"#).await;
        if let super::LogEntry::PlayerLeftBuyzone {
            prefix,
            player,
            instruments,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(player.nick, "foobar");
            assert_eq!(player.entity_index, 20);
            assert_eq!(
                player.id,
                super::PlayerID::STAMID("STEAM_1:1:12345678".to_string())
            );
            assert_eq!(player.team, super::TeamAll::CT);
            assert_eq!(instruments.len(), 6);
            assert_eq!(true, instruments.contains(&"weapon_knife".to_string()));
            assert_eq!(true, instruments.contains(&"weapon_hkp2000".to_string()));
            assert_eq!(true, instruments.contains(&"weapon_famas".to_string()));
            assert_eq!(true, instruments.contains(&"weapon_hegrenade".to_string()));
            assert_eq!(true, instruments.contains(&"kevlar(100)".to_string()));
            assert_eq!(true, instruments.contains(&"helmet".to_string()));
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_team_chat() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: "foobar<20><STEAM_1:1:12345678><CT>" say_team "hello world!""#).await;
        if let super::LogEntry::TeamChat {
            prefix,
            player,
            msg,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(msg, "hello world!");
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_money_changed() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: "foobar<20><STEAM_1:1:12345678><CT>" money change 12600-1050 = $11550 (tracked) (purchase: weapon_mac10)"#).await;
        if let super::LogEntry::MoneyChanged {
            prefix,
            player,
            previously,
            operation,
            change,
            new_amount,
            tracked,
            instrument,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(player.nick, "foobar");
            assert_eq!(player.entity_index, 20);
            assert_eq!(
                player.id,
                super::PlayerID::STAMID("STEAM_1:1:12345678".to_string())
            );
            assert_eq!(player.team, super::TeamAll::CT);
            assert_eq!(previously, 12600);
            assert_eq!(operation, "-");
            assert_eq!(change, 1050);
            assert_eq!(new_amount, 11550);
            assert_eq!(tracked, true);
            assert_eq!(instrument, Some("weapon_mac10".to_string()));
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_player_purchased() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: "foobar<20><STEAM_1:1:12345678><CT>" purchased "item_assaultsuit""#).await;
        if let super::LogEntry::PlayerPurchased {
            prefix,
            player,
            instrument,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(player.nick, "foobar");
            assert_eq!(player.entity_index, 20);
            assert_eq!(
                player.id,
                super::PlayerID::STAMID("STEAM_1:1:12345678".to_string())
            );
            assert_eq!(player.team, super::TeamAll::CT);
            assert_eq!(instrument, "item_assaultsuit");
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_threw_flashbang() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: "foobar<20><STEAM_1:1:12345678><CT>" threw flashbang [-510 2234 -103] flashbang entindex 333)"#).await;
        if let super::LogEntry::ThrewFlashbang {
            prefix,
            player,
            location,
            entindex,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(player.nick, "foobar");
            assert_eq!(player.entity_index, 20);
            assert_eq!(
                player.id,
                super::PlayerID::STAMID("STEAM_1:1:12345678".to_string())
            );
            assert_eq!(player.team, super::TeamAll::CT);
            assert_eq!(location.x, -510);
            assert_eq!(location.y, 2234);
            assert_eq!(location.z, -103);
            assert_eq!(entindex, 333);
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_blinded_player() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: "foobar<20><STEAM_1:1:12345678><CT>" blinded for 0.68 by "bazgaz<10><STEAM_1:1:87654321><TERRORIST>" from flashbang entindex 333 "#).await;
        if let super::LogEntry::BlindedPlayer {
            prefix,
            offender,
            duration,
            victim,
            entindex,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(offender.nick, "foobar");
            assert_eq!(offender.entity_index, 20);
            assert_eq!(
                offender.id,
                super::PlayerID::STAMID("STEAM_1:1:12345678".to_string())
            );
            assert_eq!(offender.team, super::TeamAll::CT);
            assert_eq!(duration, Duration::from_millis(680));
            assert_eq!(victim.nick, "bazgaz");
            assert_eq!(victim.entity_index, 10);
            assert_eq!(
                victim.id,
                super::PlayerID::STAMID("STEAM_1:1:87654321".to_string())
            );
            assert_eq!(victim.team, super::TeamAll::TERRORIST);
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_global_chat() {
        let logentry = parse_line(
            r#"L 01/02/2020 - 03:04:05: "foobar<20><STEAM_1:1:12345678><CT>" say "!ready""#,
        )
        .await;
        if let super::LogEntry::GlobalChat {
            prefix,
            player,
            msg,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(player.nick, "foobar");
            assert_eq!(player.entity_index, 20);
            assert_eq!(
                player.id,
                super::PlayerID::STAMID("STEAM_1:1:12345678".to_string())
            );
            assert_eq!(player.team, super::TeamAll::CT);
            assert_eq!(msg, "!ready");
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_player_killed_entity() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: "foobar<20><STEAM_1:1:12345678><CT>" [-2578 322 461] killed other "func_breakable<440>" [-1706 1406 666] with "ak47" (penetrated)"#).await;
        if let super::LogEntry::PlayerKilledEntity {
            prefix,
            player,
            location,
            entity_name,
            entindex,
            entity_location,
            instrument,
            kill_attributes,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(player.nick, "foobar");
            assert_eq!(player.entity_index, 20);
            assert_eq!(
                player.id,
                super::PlayerID::STAMID("STEAM_1:1:12345678".to_string())
            );
            assert_eq!(player.team, super::TeamAll::CT);
            assert_eq!(location.x, -2578);
            assert_eq!(location.y, 322);
            assert_eq!(location.z, 461);
            assert_eq!(entity_name, "func_breakable");
            assert_eq!(entindex, 440);
            assert_eq!(entity_location.x, -1706);
            assert_eq!(entity_location.y, 1406);
            assert_eq!(entity_location.z, 666);
            assert_eq!(instrument, "ak47");
            assert_eq!(
                kill_attributes,
                KillAttributes {
                    headshot: false,
                    penetrated: true
                }
            );
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_player_killed_player() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: "foobar<20><STEAM_1:1:12345678><CT>" [-2563 -1378 434] killed "bazgaz<10><STEAM_1:1:87654321><TERRORIST>" [-2307 -1025 457] with "glock" (headshot)"#).await;
        if let super::LogEntry::PlayerKilledPlayer {
            prefix,
            offender,
            offender_location,
            victim,
            victim_location,
            instrument,
            kill_attributes,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(offender.nick, "foobar");
            assert_eq!(offender.entity_index, 20);
            assert_eq!(
                offender.id,
                super::PlayerID::STAMID("STEAM_1:1:12345678".to_string())
            );
            assert_eq!(offender.team, super::TeamAll::CT);
            assert_eq!(offender_location.x, -2563);
            assert_eq!(offender_location.y, -1378);
            assert_eq!(offender_location.z, 434);
            assert_eq!(victim.nick, "bazgaz");
            assert_eq!(victim.entity_index, 10);
            assert_eq!(
                victim.id,
                super::PlayerID::STAMID("STEAM_1:1:87654321".to_string())
            );
            assert_eq!(victim.team, super::TeamAll::TERRORIST);
            assert_eq!(victim_location.x, -2307);
            assert_eq!(victim_location.y, -1025);
            assert_eq!(victim_location.z, 457);
            assert_eq!(instrument, "glock");
            assert_eq!(
                kill_attributes,
                KillAttributes {
                    headshot: true,
                    penetrated: false
                }
            );
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_player_threw_smokegrenade() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: "foobar<20><STEAM_1:1:12345678><CT>" threw smokegrenade [-300 1480 -123]"#).await;
        if let super::LogEntry::PlayerThrewSmokegrenade {
            prefix,
            player,
            location,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(player.nick, "foobar");
            assert_eq!(player.entity_index, 20);
            assert_eq!(
                player.id,
                super::PlayerID::STAMID("STEAM_1:1:12345678".to_string())
            );
            assert_eq!(player.team, super::TeamAll::CT);
            assert_eq!(location.x, -300);
            assert_eq!(location.y, 1480);
            assert_eq!(location.z, -123);
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_player_threw_hegrenade() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: "foobar<20><STEAM_1:1:12345678><CT>" threw hegrenade [671 510 38]"#).await;
        if let super::LogEntry::PlayerThrewHEGrenade {
            prefix,
            player,
            location,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(player.nick, "foobar");
            assert_eq!(player.entity_index, 20);
            assert_eq!(
                player.id,
                super::PlayerID::STAMID("STEAM_1:1:12345678".to_string())
            );
            assert_eq!(player.team, super::TeamAll::CT);
            assert_eq!(location.x, 671);
            assert_eq!(location.y, 510);
            assert_eq!(location.z, 38);
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_player_attacked_player() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: "foobar<20><STEAM_1:1:12345678><CT>" [606 2152 -98] attacked "bazgaz<10><STEAM_1:1:87654321><TERRORIST>" [334 2434 -120] with "ak47" (damage "141") (damage_armor "0") (health "0") (armor "0") (hitgroup "head")"#).await;
        if let super::LogEntry::PlayerAttackedPlayer {
            prefix,
            offender,
            offender_location,
            victim,
            victim_location,
            instrument,
            damage,
            damage_armor,
            health,
            armor,
            hitgroup,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(offender.nick, "foobar");
            assert_eq!(offender.entity_index, 20);
            assert_eq!(
                offender.id,
                super::PlayerID::STAMID("STEAM_1:1:12345678".to_string())
            );
            assert_eq!(offender.team, super::TeamAll::CT);
            assert_eq!(offender_location.x, 606);
            assert_eq!(offender_location.y, 2152);
            assert_eq!(offender_location.z, -98);
            assert_eq!(victim.nick, "bazgaz");
            assert_eq!(victim.entity_index, 10);
            assert_eq!(
                victim.id,
                super::PlayerID::STAMID("STEAM_1:1:87654321".to_string())
            );
            assert_eq!(victim.team, super::TeamAll::TERRORIST);
            assert_eq!(victim_location.x, 334);
            assert_eq!(victim_location.y, 2434);
            assert_eq!(victim_location.z, -120);
            assert_eq!(instrument, "ak47");
            assert_eq!(damage, 141);
            assert_eq!(damage_armor, 0);
            assert_eq!(health, 0);
            assert_eq!(armor, 0);
            assert_eq!(hitgroup, HitGroup::Head);
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_player_disconnected() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: "foobar<20><STEAM_1:1:12345678><CT>" disconnected (reason "Disconnect")"#).await;
        if let super::LogEntry::PlayerDisconnected {
            prefix,
            player,
            reason,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(player.nick, "foobar");
            assert_eq!(player.entity_index, 20);
            assert_eq!(
                player.id,
                super::PlayerID::STAMID("STEAM_1:1:12345678".to_string())
            );
            assert_eq!(player.team, super::TeamAll::CT);
            assert_eq!(reason, "Disconnect");
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_player_assisted_killing_player() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: "foobar<20><STEAM_1:1:12345678><CT>" assisted killing "bazgaz<10><STEAM_1:1:87654321><TERRORIST>""#).await;
        if let super::LogEntry::PlayerAssistedKillingPlayer {
            prefix,
            offender,
            victim,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(offender.nick, "foobar");
            assert_eq!(offender.entity_index, 20);
            assert_eq!(
                offender.id,
                super::PlayerID::STAMID("STEAM_1:1:12345678".to_string())
            );
            assert_eq!(offender.team, super::TeamAll::CT);
            assert_eq!(victim.nick, "bazgaz");
            assert_eq!(victim.entity_index, 10);
            assert_eq!(
                victim.id,
                super::PlayerID::STAMID("STEAM_1:1:87654321".to_string())
            );
            assert_eq!(victim.team, super::TeamAll::TERRORIST);
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_player_assisted_blinding_player() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: "foobar<20><STEAM_1:1:12345678><CT>" flash-assisted killing "bazgaz<10><STEAM_1:1:87654321><TERRORIST>""#).await;
        if let super::LogEntry::PlayerAssistedBlindingPlayer {
            prefix,
            offender,
            victim,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(offender.nick, "foobar");
            assert_eq!(offender.entity_index, 20);
            assert_eq!(
                offender.id,
                super::PlayerID::STAMID("STEAM_1:1:12345678".to_string())
            );
            assert_eq!(offender.team, super::TeamAll::CT);
            assert_eq!(victim.nick, "bazgaz");
            assert_eq!(victim.entity_index, 10);
            assert_eq!(
                victim.id,
                super::PlayerID::STAMID("STEAM_1:1:87654321".to_string())
            );
            assert_eq!(victim.team, super::TeamAll::TERRORIST);
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_spawned_molotov() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: Molotov projectile spawned at 1607.403809 -1526.890625 -341.364044, velocity -812.841064 -28.768530 418.548157"#).await;
        if let super::LogEntry::SpawnedMolotov {
            prefix,
            location_x,
            location_y,
            location_z,
            velocity_x,
            velocity_y,
            velocity_z,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(location_x, 1607.403809);
            assert_eq!(location_y, -1526.890625);
            assert_eq!(location_z, -341.364044);
            assert_eq!(velocity_x, -812.841064);
            assert_eq!(velocity_y, -28.768530);
            assert_eq!(velocity_z, 418.548157);
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_threw_molotov() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: "foobar<20><STEAM_1:1:12345678><CT>" threw molotov [-84 1372 106]"#).await;
        if let super::LogEntry::ThrewMolotov {
            prefix,
            player,
            location,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(player.nick, "foobar");
            assert_eq!(player.entity_index, 20);
            assert_eq!(
                player.id,
                super::PlayerID::STAMID("STEAM_1:1:12345678".to_string())
            );
            assert_eq!(player.team, super::TeamAll::CT);
            assert_eq!(location.x, -84);
            assert_eq!(location.y, 1372);
            assert_eq!(location.z, 106);
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_player_connected() {
        let logentry = parse_line(
            r#"L 01/02/2020 - 03:04:05: "foobar<20><STEAM_1:1:12345678><>" connected, address """#,
        )
        .await;
        if let super::LogEntry::PlayerConnected {
            prefix,
            player,
            address,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(player.nick, "foobar");
            assert_eq!(player.entity_index, 20);
            assert_eq!(
                player.id,
                super::PlayerID::STAMID("STEAM_1:1:12345678".to_string())
            );
            assert_eq!(player.team, super::TeamAll::UNASSIGNED);
            assert_eq!(address, "");
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_validated_steamid() {
        let logentry = parse_line(
            r#"L 01/02/2020 - 03:04:05: "foobar<20><STEAM_1:1:12345678><>" STEAM USERID validated"#,
        )
        .await;
        if let super::LogEntry::ValidatedSteamID { prefix, player } = logentry {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(player.nick, "foobar");
            assert_eq!(player.entity_index, 20);
            assert_eq!(
                player.id,
                super::PlayerID::STAMID("STEAM_1:1:12345678".to_string())
            );
            assert_eq!(player.team, super::TeamAll::UNASSIGNED);
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_team_scored() {
        let logentry =
            parse_line(r#"L 01/02/2020 - 03:04:05: Team "CT" scored "0" with "5" players"#).await;
        if let super::LogEntry::TeamScored {
            prefix,
            team,
            score,
            player_count,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(team, Team::CT);
            assert_eq!(score, 0);
            assert_eq!(player_count, 5);
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_threw_decoy() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: "foobar<20><STEAM_1:1:12345678><CT>" threw decoy [-427 1737 -126]"#).await;
        if let super::LogEntry::ThrewDecoy {
            prefix,
            player,
            location,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(player.nick, "foobar");
            assert_eq!(player.entity_index, 20);
            assert_eq!(
                player.id,
                super::PlayerID::STAMID("STEAM_1:1:12345678".to_string())
            );
            assert_eq!(player.team, super::TeamAll::CT);
            assert_eq!(location.x, -427);
            assert_eq!(location.y, 1737);
            assert_eq!(location.z, -126);
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_match_resumed() {
        let logentry =
            parse_line(r#"L 01/02/2020 - 03:04:05: Match pause is disabled - mp_unpause_match"#)
                .await;
        if let super::LogEntry::MatchResumed { prefix } = logentry {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_match_paused() {
        let logentry =
            parse_line(r#"L 01/02/2020 - 03:04:05: Match pause is enabled - mp_pause_match"#).await;
        if let super::LogEntry::MatchPaused { prefix } = logentry {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_killed_by_bomb() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: "foobar<20><STEAM_1:1:12345678><CT>" [2410 -382 147] was killed by the bomb."#).await;
        if let super::LogEntry::KilledByBomb {
            prefix,
            player,
            location,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(player.nick, "foobar");
            assert_eq!(player.entity_index, 20);
            assert_eq!(
                player.id,
                super::PlayerID::STAMID("STEAM_1:1:12345678".to_string())
            );
            assert_eq!(player.team, super::TeamAll::CT);
            assert_eq!(location.x, 2410);
            assert_eq!(location.y, -382);
            assert_eq!(location.z, 147);
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_accolade() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: ACCOLADE, FINAL: {assists},    foobar<2>,      VALUE: 3.000000,        POS: 1, SCORE: 0.357143"#).await;
        if let super::LogEntry::Accolade {
            prefix,
            categorie,
            player,
            player_entindex,
            value,
            pos,
            score,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(categorie, "assists");
            assert_eq!(player, "foobar");
            assert_eq!(player_entindex, 2);
            assert_eq!(value, 3.0);
            assert_eq!(pos, 1);
            assert_eq!(score, 0.357143);
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_game_over() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: Game Over: competitive mg_active de_inferno score 11:16 after 50 min"#).await;
        if let super::LogEntry::GameOver {
            prefix,
            mode,
            map_group,
            map,
            ct_score,
            t_score,
            time,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(mode, "competitive");
            assert_eq!(map_group, "mg_active");
            assert_eq!(map, "de_inferno");
            assert_eq!(ct_score, 11);
            assert_eq!(t_score, 16);
            assert_eq!(time, Duration::from_secs(50 * 60));
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_changed_nickname() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: "foobar<20><STEAM_1:1:12345678><CT>" changed name to "bazgaz""#).await;
        if let super::LogEntry::ChangedNickname {
            prefix,
            player,
            new_nickname,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(player.nick, "foobar");
            assert_eq!(player.entity_index, 20);
            assert_eq!(
                player.id,
                super::PlayerID::STAMID("STEAM_1:1:12345678".to_string())
            );
            assert_eq!(player.team, super::TeamAll::CT);
            assert_eq!(new_nickname, "bazgaz");
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_committed_suicide() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: "foobar<20><STEAM_1:1:12345678><CT>" [258 2481 -57] committed suicide with "world""#).await;
        if let super::LogEntry::CommittedSuicide {
            prefix,
            player,
            location,
            instrument,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(player.nick, "foobar");
            assert_eq!(player.entity_index, 20);
            assert_eq!(
                player.id,
                super::PlayerID::STAMID("STEAM_1:1:12345678".to_string())
            );
            assert_eq!(player.team, super::TeamAll::CT);
            assert_eq!(location.x, 258);
            assert_eq!(location.y, 2481);
            assert_eq!(location.z, -57);
            assert_eq!(instrument, "world");
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_server_message() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: server_message: "quit""#).await;
        if let super::LogEntry::ServerMessage { prefix, message } = logentry {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(message, "quit");
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_steam_auth_failure() {
        let logentry = parse_line(
            r#"L 01/02/2020 - 03:04:05: STEAMAUTH: Client foo bar received failure code 6"#,
        )
        .await;
        if let super::LogEntry::SteamAuthFailure {
            prefix,
            nickname,
            failure_code,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(nickname, "foo bar"); // Intentional space in the nickname, the regex should capture it correctly
            assert_eq!(failure_code, 6);
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_meta_mod_plugins_loaded_1() {
        let logentry =
            parse_line(r#"L 01/02/2020 - 03:04:05: [META] Loaded 0 plugins (1 already loaded)"#)
                .await;
        if let super::LogEntry::MetaModPluginsLoaded {
            prefix,
            loaded,
            preloaded,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(loaded, 0);
            assert_eq!(preloaded, 1);
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[actix_rt::test]
    async fn log_meta_mod_plugins_loaded_2() {
        let logentry = parse_line(r#"L 01/02/2020 - 03:04:05: [META] Loaded 1 plugin."#).await;
        if let super::LogEntry::MetaModPluginsLoaded {
            prefix,
            loaded,
            preloaded,
        } = logentry
        {
            assert_eq!(prefix.month, 1);
            assert_eq!(prefix.day, 2);
            assert_eq!(prefix.year, 2020);
            assert_eq!(prefix.hour, 3);
            assert_eq!(prefix.minute, 4);
            assert_eq!(prefix.second, 5);
            assert_eq!(loaded, 1);
            assert_eq!(preloaded, 0);
        } else {
            panic!("wrong LogEntry type received, {:#?}", logentry)
        }
    }

    #[test]
    #[ignore]
    /// Process a pile of CS:GO log files, and check if all lines can be matched.
    /// Ignored by default because not all developers have access to logs to test on.
    fn parse_log_files() {
        // Fill with file paths to log files
        let files: Vec<&str> = vec![
            // Insert paths to log files
        ];

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
                    Err(err) => panic!(err),
                }
            }
        }
    }
}
