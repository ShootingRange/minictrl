table! {
    maplist (id) {
        id -> Int4,
        order -> Int4,
        map -> Varchar,
    }
}

table! {
    match (id) {
        id -> Int4,
        server_id -> Int4,
        team1_id -> Int4,
        team2_id -> Int4,
        team1_score -> Nullable<Int4>,
        team2_score -> Nullable<Int4>,
        num_maps -> Int4,
        skip_veto -> Bool,
        veto_first -> Side_type,
        players_per_team -> Int4,
        min_player_to_ready -> Int4,
    }
}

table! {
    match_spectator (match_id) {
        match_id -> Int4,
        spectator_id -> Int4,
    }
}

table! {
    player (id) {
        id -> Int4,
        team_id -> Int4,
        name -> Nullable<Varchar>,
        tag -> Nullable<Varchar>,
        steamid -> Nullable<Varchar>,
    }
}

table! {
    server (id) {
        id -> Int4,
        host -> Inet,
        port -> Int2,
        #[sql_name = "type"]
        type_ -> Nullable<Varchar>,
    }
}

table! {
    spectator (id) {
        id -> Int4,
        steamid -> Varchar,
    }
}

table! {
    team (id) {
        id -> Int4,
        name -> Varchar,
        country -> Nullable<Bpchar>,
        logo -> Nullable<Varchar>,
    }
}

joinable!(match_spectator -> match (match_id));
joinable!(match_spectator -> spectator (spectator_id));
joinable!(player -> team (team_id));

allow_tables_to_appear_in_same_query!(
    maplist,
    match,
    match_spectator,
    player,
    server,
    spectator,
    team,
);
