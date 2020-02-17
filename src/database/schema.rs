table! {
    maplist (id) {
        id -> Int4,
        match_id -> Int4,
        order -> Int4,
        map -> Varchar,
    }
}

table! {
    match_spectator (match_id) {
        id -> Int4,
        match_id -> Int4,
        spectator_id -> Int4,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::common::Side;

    matches (id) {
        id -> Int4,
        server_id -> Int4,
        team1_id -> Int4,
        team2_id -> Int4,
        team1_score -> Nullable<Int4>,
        team2_score -> Nullable<Int4>,
        num_maps -> Int4,
        skip_veto -> Bool,
        veto_first -> Side,
        players_per_team -> Int4,
        min_player_to_ready -> Int4,
    }
}

table! {
    players (id) {
        id -> Int4,
        name -> Varchar,
        team_id -> Int4,
        tag -> Nullable<Varchar>,
        steamid -> Nullable<Varchar>,
    }
}

table! {
    servers (id) {
        id -> Int4,
        host -> Inet,
        port -> Int2,
        #[sql_name = "type"]
        type_ -> Nullable<Varchar>,
    }
}

table! {
    spectators (id) {
        id -> Int4,
        steamid -> Varchar,
    }
}

table! {
    teams (id) {
        id -> Int4,
        name -> Varchar,
        country -> Nullable<Bpchar>,
        logo -> Nullable<Varchar>,
    }
}

joinable!(maplist -> matches (match_id));
joinable!(match_spectator -> matches (match_id));
joinable!(match_spectator -> spectators (spectator_id));
joinable!(players -> teams (team_id));

allow_tables_to_appear_in_same_query!(
    maplist,
    match_spectator,
    matches,
    players,
    servers,
    spectators,
    teams,
);
