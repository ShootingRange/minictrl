use regex::RegexSet;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::Path;

fn parse_logfile<P: AsRef<Path>>(path: P) {
    // Prefix for a CS:GO log line.
    let prefix = r"^L (?P<log_month>\d\d)/(?P<log_day>\d\d)/(?P<log_year>\d\d\d\d) - (?P<log_hour>\d\d):(?P<log_minute>\d\d):(?P<log_second>\d\d): ".to_string();

    // TODO player regex fails on players with a "<" character in their name

    let rset = RegexSet::new([
        // Start of log file.
        prefix.clone() + r#"Log file started \(file "(?P<file>[^"]*)"\) \(game "(?P<game>[^"]*)"\) \(version "(?P<version>\d+)"\)$"#,
        // End of log file.
        prefix.clone() + r#"Log file closed$"#,
        // World triggered game event.
        prefix.clone() + r#"World triggered "(?P<event>[^"]*)"$"#,
        // World triggered game event in relation to map.
        // Only seen with event "Match_Start".
        prefix.clone() + r#"World triggered "(?P<event>[^"]*)" on "(?P<map>[^"]*)"$"#,
        // World triggered event with meta information about team scores.
        // Only seen with event "SFUI_Notice_Round_Draw".
        prefix.clone() + r#"World triggered "(?P<event>[^"]*)" \(CT "(?P<ct>\d+)"\) \(T "(?P<t>\d+)"\)$"#,
        // Player triggered game event.
        prefix.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" triggered "(?P<event>[^"]*)"$"#,
        // Team triggered game event with meta information about team scores.
        prefix.clone() + r#"Team "(?P<team>(TERRORIST|CT))" triggered "(?P<event>[^"]*)" \(CT "(?P<ct>\d+)"\) \(T "(?P<t>\d+)"\)$"#,
        // Loading map.
        prefix.clone() + r#"Loading map "(?P<map>[^"]*)"$"#,
        // Server started dumping cvars.
        prefix.clone() + r#"server cvars start$"#,
        // Individual cvar from cvars dump.
        prefix.clone() + r#""(?P<cvar_key>[^"]*)" = "(?P<cvar_value>[^"]*)"$"#,
        // Server ended cvars dump.
        prefix.clone() + r#"server cvars end$"#,
        // Started map.
        prefix.clone() + r#"Started map "(?P<map>[^"]*)" \(CRC "(?P<crc>-?\d+)"\)$"#,
        // Server emitted a single cvar.
        prefix.clone() + r#"server_cvar: "(?P<cvar_key>[^"]*)" "(?P<cvar_value>[^"]*)"$"#,
        // Player entered the game.
        // Team field is always empty.
        prefix.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" entered the game$"#,
        // Get5 event encoded as JSON.
        prefix.clone() + r#"get5_event: (?P<json>.+)$"#,
        // Command was executed over RCON.
        // The command can contain double quotes, take care when editing the regex.
        prefix.clone() + r#"rcon from "(?P<ip_addres>\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}:\d{1,5})": command "(?P<command>.*)"$"#,
        // Bad password during RCON authentication.
        prefix.clone() + r#"rcon from "(?P<ip_addres>\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}:\d{1,5})": Bad Password$"#,
        // Player switched from on team/side to another.
        prefix.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT))>" switched from team <(?P<from_side>(Unassigned|TERRORIST|CT|Spectator)?)> to <(?P<to_side>(Unassigned|TERRORIST|CT|Spectator)?)>$"#,
        // Player picked up instrument/equipment.
        prefix.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" picked up "(?P<instrument>[^"]*)"$"#,
        // Player dropped instrument/equipment.
        prefix.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" dropped "(?P<instrument>[^"]*)"$"#,
        // TODO description
        prefix.clone() + r#"Team playing "(?P<side>(CT|TERRORIST))": (\[(?P<readiness>(NOT )?READY)\] )?(?P<team>.*)$"#,
        // Freeze period started.
        prefix.clone() + r#"Starting Freeze period$"#,
        // Player left buyzone, and can no longer buy equipment until next round.
        prefix.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" left buyzone with \[ (?P<instruments>([A-Za-z0-9_]*(\(\d+\))? )*)\]$"#,
        // Player send message in team chat.
        prefix.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" say_team "(?P<msg>.*)"$"#,
        // Player's money changed.
        // Can be caused by round change, or purchase. It's not known what the "tracked" attribute indicates. The resulting money may be capped by `mp_maxmoney`.
        prefix.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" money change (?P<money_prev>\d+)(?P<moeny_op>[\+-])(?P<money_diff>\d+) = \$(?P<money_after>\d+)( \((?P<tracked>tracked)\)( \(purchase: (?P<instrument>[A-Za-z0-9_]*(\(\d+\))?)\))?)?$"#,
        // Player purchased instrument/equipment.
        prefix.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" purchased "(?P<instrument>[A-Za-z0-9_]*(\(\d+\))?)"$"#,
        // Player threw flashbang.
        prefix.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" threw flashbang \[(?P<loc_x>-?\d+) (?P<loc_y>-?\d+) (?P<loc_z>-?\d+)\] flashbang entindex (?P<entindex>\d+)\)$"#, // Extra bracket at end
        // Player was blinded by flashbang thrown by another player
        prefix.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" blinded for \d+\.\d{2} by "(?P<p2_nick>[^<]*)<(?P<p2_entindex>\d+)><(?P<p2_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<p2_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" from flashbang entindex (?P<entindex>\d+) $"#, // Trailing space
        // Player send message in global chat. Both teams will see these messages.
        prefix.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" say "(?P<msg>.*)"$"#, // Message can contain double quotes
        // Player killed entity
        prefix.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" \[(?P<loc_x>-?\d+) (?P<loc_y>-?\d+) (?P<loc_z>-?\d+)\] killed other "[^<]*<\d+>" \[(?P<ent_x>-?\d+) (?P<ent_y>-?\d+) (?P<ent_z>-?\d+)\] with "(?P<instrument>[A-Za-z0-9_]*(\(\d+\))?)"( \((?P<kill_attributes>(headshot|penetrated|headshot penetrated))\))?$"#,
        // Player killed another player with instrument
        prefix.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" \[(?P<loc_x>-?\d+) (?P<loc_y>-?\d+) (?P<loc_z>-?\d+)\] killed "(?P<p2_nick>[^<]*)<(?P<p2_entindex>\d+)><(?P<p2_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<p2_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" \[(?P<p2_x>-?\d+) (?P<p2_y>-?\d+) (?P<p2_z>-?\d+)\] with "(?P<instrument>[A-Za-z0-9_]*(\(\d+\))?)"( \((?P<kill_attributes>(headshot|penetrated|headshot penetrated))\))?$"#,
        // Player threw smokegrenade
        prefix.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" threw smokegrenade \[(?P<loc_x>-?\d+) (?P<loc_y>-?\d+) (?P<loc_z>-?\d+)\]$"#,
        // Player threw high explosive grenade.
        prefix.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" threw hegrenade \[(?P<loc_x>-?\d+) (?P<loc_y>-?\d+) (?P<loc_z>-?\d+)\]$"#,
        // Player attacked another player
        prefix.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" \[(?P<p1_x>-?\d+) (?P<p1_y>-?\d+) (?P<p1_z>-?\d+)\] attacked "(?P<player_attacked_nick>[^<]*)<(?P<player_attacked_entindex>\d+)><(?P<player_attacked_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_attacked_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" \[(?P<p2_x>-?\d+) (?P<p2_y>-?\d+) (?P<p2_z>-?\d+)\] with "(?P<instrument>[A-Za-z0-9_]*(\(\d+\))?)" \(damage "(?P<damage>\d+)"\) \(damage_armor "(?P<damage_armor>\d+)"\) \(health "(?P<health>\d+)"\) \(armor "(?P<armor>\d+)"\) \(hitgroup "(?P<hitgroup>(chest|generic|head|left arm|left leg|neck|right arm|right leg|stomach))"\)$"#,
        // Player disconnected from game server.
        prefix.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" disconnected \(reason "(?P<reason>[^"]*)"\)$"#,
        // Player assisted another player in killing a third player.
        prefix.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" assisted killing "(?P<player_killed_nick>[^<]*)<(?P<player_killed_entindex>\d+)><(?P<player_killed_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_killed_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>"$"#,
        // Player assisted another player in killing a third player by blinding them.
        prefix.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" flash-assisted killing "(?P<player_killed_nick>[^<]*)<(?P<player_killed_entindex>\d+)><(?P<player_killed_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_killed_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>"$"#,
        // Molotov projectile spawned
        prefix.clone() + r#"Molotov projectile spawned at (?P<loc_x>-?\d+\.\d+) (?P<loc_y>-?\d+\.\d+) (?P<loc_z>-?\d+\.\d+), velocity (?P<vec_x>-?\d+\.\d+) (?P<vec_y>-?\d+\.\d+) (?P<vec_z>-?\d+\.\d+)$"#,
        // Player threw molotov
        prefix.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" threw molotov \[(?P<loc_x>-?\d+) (?P<loc_y>-?\d+) (?P<loc_z>-?\d+)\]$"#,
        // Player conncted to game server.
        prefix.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" connected, address "(?P<ip_address>[^"]*)"$"#,
        // SteamID of player was validated.
        prefix.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" STEAM USERID validated$"#,
        // Team ended match with given score, and number of player participating.
        prefix.clone() + r#"Team "(?P<side>(CT|TERRORIST))" scored "(?P<score>\d+)" with "(?P<player_count>\d+)" players$"#,
        // Player threw decoy.
        prefix.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" threw decoy \[(?P<loc_x>-?\d+) (?P<loc_y>-?\d+) (?P<loc_z>-?\d+)\]$"#,
        // Match paused.
        prefix.clone() + r#"Match pause is disabled - mp_unpause_match$"#,
        // Match pause ended.
        prefix.clone() + r#"Match pause is enabled - mp_pause_match$"#,
        // Player was killed by bomb
        prefix.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" \[(?P<loc_x>-?\d+) (?P<loc_y>-?\d+) (?P<loc_z>-?\d+)\] was killed by the bomb\.$"#,
        // TODO description
        prefix.clone() + r#"ACCOLADE, FINAL: \{[^\}]*\},\s+[^<]*<\d+>,\s+VALUE: \d+\.\d+,\s+POS: \d+,\s+SCORE: \d+\.\d+$"#,
        // Game ended.
        // CT and T score might be swapped, this needs validation.
        prefix.clone() + r#"Game Over: (?P<mode>[A-Za-z0-9_]+) (?P<map_group>[A-Za-z0-9_]+) (?P<map>[A-Za-z0-9_]+) score (?P<ct_score>\d+):(?P<t_score>\d+) after (?P<time>\d+) min$"#,
        // Player changed their nickname.
        prefix.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" changed name to "(?P<new_nick>[^"]*)"$"#,
        // Player commited suicide with instrument
        prefix.clone() + r#""(?P<player_nick>[^<]*)<(?P<player_entindex>\d+)><(?P<player_id>(STEAM_\d:\d:\d+|BOT|Console))><(?P<player_team>(Unassigned|TERRORIST|CT|Spectator|Console)?)>" \[(?P<loc_x>-?\d+) (?P<loc_y>-?\d+) (?P<loc_z>-?\d+)\] committed suicide with "(?P<instrument>[^"]*)"$"#,
        // TODO description
        prefix.clone() + r#"server_message: "(?P<msg>[^"]*)"$"#,
        // Failed to validate user authentication ticket.
        // Error codes are described in the [`steam_api.h` documentation](https://partner.steamgames.com/doc/api/steam_api#EAuthSessionResponse).
        prefix.clone() + r#"STEAMAUTH: Client (?P<player_nick>.*) received failure code (?P<code>\d+)$"#,
        // META mod has loaded plugins.
        prefix.clone() + r#"\[META\] Loaded (?P<plugins_loaded>\d+) plugin(s|\.)( \((?P<plugins_preloaded>\d+) already loaded\))?$"#,
    ].iter()).unwrap();

    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        match line {
            Ok(line) => {
                let m = rset.matches(&line);
                if !m.matched_any() {
                    println!("{}", line);
                }
            },
            Err(err) => {
                panic!(err)
            },
        }
    }
}
