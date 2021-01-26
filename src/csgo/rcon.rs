use get5status::Get5Status;
use rcon::Connection;

// enum definition: https://github.com/splewis/get5/blob/51fe79d0da8131f7104e4a78551f4364f06be950/scripting/include/get5.inc#L6
// serialization code: https://github.com/splewis/get5/blob/d5dd9f8fa501261cd2f15067d55b1f7f25e1530b/scripting/get5.sp#L1324
// TODO maps: json object, key "map%d", value: map name, https://github.com/splewis/get5/blob/51fe79d0da8131f7104e4a78551f4364f06be950/scripting/get5.sp#L1357
pub mod get5status {
    use std::collections::HashMap;

    #[derive(Deserialize, Debug)]
    //#[serde(tag = "gamestate")] // The serde rename macro doesn't support integers, see https://github.com/serde-rs/serde/pull/1392
    #[serde(tag = "gamestate_string")]
    pub enum Get5Status {
        //#[serde(rename = "0")]
        #[serde(rename = "none")]
        Idle {
            //gamestate_string: String,
            plugin_version: String,
            paused: bool,
        },

        //#[serde(rename = "1")]
        #[serde(rename = "waiting for map veto")]
        PreVeto {
            matchid: String,
            //gamestate_string: String,
            loaded_config_file: String,
            plugin_version: String,
            map_number: i32,
            team2: Team,
            team1: Team,
            paused: bool,
        },

        //#[serde(rename = "2")]
        #[serde(rename = "map veto")]
        Veto {
            matchid: String,
            //gamestate_string: String,
            loaded_config_file: String,
            plugin_version: String,
            map_number: i32,
            team2: Team,
            team1: Team,
            paused: bool,
            maps: HashMap<String, String>,
            // TODO `maps`
        },

        //#[serde(rename = "3")]
        #[serde(rename = "warmup")]
        Warmup {
            matchid: String,
            //gamestate_string: String,
            loaded_config_file: String,
            plugin_version: String,
            map_number: i32,
            team2: Team,
            team1: Team,
            paused: bool,
            maps: HashMap<String, String>,
            // TODO `maps`
        },

        //#[serde(rename = "4")]
        #[serde(rename = "knife round")]
        KnifeRound {
            matchid: String,
            //gamestate_string: String,
            loaded_config_file: String,
            plugin_version: String,
            map_number: i32,
            team2: Team,
            team1: Team,
            paused: bool,
            maps: HashMap<String, String>,
            // TODO `maps`
        },

        //#[serde(rename = "5")]
        #[serde(rename = "waiting for knife round decision")]
        WaitingForKnifeRoundDecision {
            matchid: String,
            //gamestate_string: String,
            loaded_config_file: String,
            plugin_version: String,
            map_number: i32,
            team2: Team,
            team1: Team,
            paused: bool,
            maps: HashMap<String, String>,
            // TODO `maps`
        },

        //#[serde(rename = "6")]
        #[serde(rename = "going live")]
        GoingLive {
            matchid: String,
            //gamestate_string: String,
            loaded_config_file: String,
            plugin_version: String,
            map_number: i32,
            team2: Team,
            team1: Team,
            paused: bool,
            maps: HashMap<String, String>,
            // TODO `maps`
        },

        //#[serde(rename = "7")]
        #[serde(rename = "live")]
        Live {
            matchid: String,
            //gamestate_string: String,
            loaded_config_file: String,
            plugin_version: String,
            map_number: i32,
            team2: Team,
            team1: Team,
            paused: bool,
            maps: HashMap<String, String>,
            // TODO `maps`
        },

        //#[serde(rename = "8")]
        #[serde(rename = "postgame")]
        PostGame {
            matchid: String,
            //gamestate_string: String,
            loaded_config_file: String,
            plugin_version: String,
            map_number: i32,
            team2: Team,
            team1: Team,
            paused: bool,
            maps: HashMap<String, String>,
            // TODO `maps`
        },
    }

    #[derive(Deserialize, Debug)]
    pub struct Team {
        connected_clients: i32,
        current_map_score: i32,
        ready: bool,
        name: String,
        series_score: i32,
        side: Side,
    }

    #[derive(Deserialize, Debug)]
    pub enum Side {
        T,
        CT,
    }
}

#[derive(Error, Debug)]
enum RCONError {
    #[error("Connection error")]
    Conn(#[from] rcon::Error),
    #[error("Could not interpret response")]
    UnexpectedReply,
    #[error("Unknown command, it is not supported by the server. A plugin might not be installed or loaded")]
    UnknownCmd,
}

async fn get5_status(conn: &mut Connection) -> Result<Get5Status, RCONError> {
    // Send command to CS:GO server
    let full_resp = conn.cmd("get5_status").await.map_err(RCONError::Conn)?;

    // Pick out the relevant line
    let reply = full_resp.lines().next().map_or(
        // Wrap in a Result so we can throw a error using the `?` operator
        Result::Err(RCONError::UnexpectedReply),
        Result::Ok,
    )?;

    if reply == "Unknown command \"get5_status\"" {
        // Get5 is not installed
        return Result::Err(RCONError::UnknownCmd);
    }

    let status: Get5Status = serde_json::from_str(reply).map_err(|_| RCONError::UnexpectedReply)?;

    Result::Ok(status)
}

#[cfg(test)]
mod tests {
    const RCON_ADDRESS: &str = "127.0.0.1:27015";
    const RCON_PASSWORD: &str = "password";

    #[tokio::test]
    async fn get5_status() {
        let mut conn = rcon::Connection::connect(RCON_ADDRESS, RCON_PASSWORD)
            .await
            .unwrap();

        match super::get5_status(&mut conn).await {
            Ok(reply) => println!("{:?}", reply),
            Err(err) => panic!("{}", err),
        };
    }
}
