CREATE TABLE teams
(
    id      serial,
    name    varchar NOT NULL,
    country char(2),
    logo    varchar,
    PRIMARY KEY (id)
);

CREATE TABLE players
(
    id      serial,
    name    varchar NOT NULL,
    team_id integer NOT NULL,
    tag     varchar,
    steamid varchar,
    FOREIGN KEY (team_id) REFERENCES teams (id),
    PRIMARY KEY (id)
);

create table servers
(
    id   serial,
    host inet               NOT NULL,
    port int DEFAULT 270.15 NOT NULL
        CONSTRAINT valid_port CHECK ( port >= 1 AND port <= 65535 ),
    type varchar,
    password text NOT NULL DEFAULT '',
    PRIMARY KEY (id)
);

CREATE TABLE spectators
(
    id      serial,
    steamid varchar NOT NULL,
    PRIMARY KEY (id)
);

CREATE TYPE side AS ENUM (
    'standard',
    'never_knife',
    'always_knife'
    );

CREATE TABLE matches
(
    id                  serial,
    server_id           integer                                                                      NOT NULL,
    team1_id            integer                                                                      NOT NULL,
    team2_id            integer                                                                      NOT NULL,
    team1_score         integer,
    team2_score         integer,
    num_maps            integer                                                                      NOT NULL,
    skip_veto           bool                                                                         NOT NULL,
    veto_first          side                                                                         NOT NULL,
    players_per_team    integer DEFAULT 5 CHECK ( players_per_team > 0 OR players_per_team IS NULL ) NOT NULL,
    min_player_to_ready integer CHECK ( min_player_to_ready <= players_per_team )                    NOT NULL,
    CHECK ( min_player_to_ready > 0 OR min_player_to_ready IS NULL ),
    PRIMARY KEY (id)
);

/* ordered list of maps to be played or voted on in a match */
CREATE TABLE maplist
(
    id       serial,
    match_id integer REFERENCES matches (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    "order"  integer                                                             NOT NULL,
    map      varchar                                                             NOT NULL, /* people could add any map to there server, and because of this it's not possible to use an enum */
    PRIMARY KEY (id),
    UNIQUE (id, "order")
);

CREATE TABLE match_spectator
(
    id           serial,
    match_id     integer REFERENCES matches (id) ON UPDATE CASCADE ON DELETE CASCADE    NOT NULL,
    spectator_id integer REFERENCES spectators (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    PRIMARY KEY (match_id)
);