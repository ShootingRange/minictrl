CREATE TABLE team
(
    id      serial,
    name    varchar not null,
    country char(2),
    logo    varchar,
    primary key (id)
);

CREATE TABLE player
(
    id      serial,
    team_id integer not null,
    name    varchar,
    tag     varchar,
    steamid varchar,
    foreign key (team_id) references team (id),
    primary key (id)
);

create table server
(
    id   serial,
    host inet not null,
    port int2 default 270.15 not null,
    type varchar,
    primary key (id)
);

create table spectator
(
    id      serial,
    steamid varchar not null,
    primary key (id)
);

create type side_type as enum (
    'standard',
    'never_knife',
    'always_knife'
);

create table "match"
(
    id                  serial,
    server_id           integer not null,
    team1_id            integer not null ,
    team2_id            integer not null,
    team1_score         integer,
    team2_score         integer,
    num_maps            integer not null,
    skip_veto           bool not null,
    veto_first          side_type not null,
    players_per_team    integer default 5 check ( players_per_team > 0 or players_per_team is null ) not null,
    min_player_to_ready integer check ( min_player_to_ready <= players_per_team ) not null,
    check ( min_player_to_ready > 0 or min_player_to_ready is null),
    primary key (id)
);

/* ordered list of maps to be played or voted on in a match */
create table maplist
(
    id      serial,
    "order" integer not null,
    map     varchar not null, /* people could add any map to there server, and because of this it's not possible to use an enum */
    primary key (id),
    unique (id, "order")
);

create table match_spectator
(
    match_id integer references match (id) on update cascade on delete cascade not null,
    spectator_id integer references spectator (id) on update cascade on delete cascade not null,
    primary key (match_id)
);