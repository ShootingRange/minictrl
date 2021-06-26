CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE teams
(
    id      uuid    NOT NULL DEFAULT uuid_generate_v4(),
    name    varchar NOT NULL,
    country char(2),
    logo    varchar,
    PRIMARY KEY (id)
);

CREATE TABLE players
(
    id      uuid                                                           NOT NULL DEFAULT uuid_generate_v4(),
    name    varchar                                                        NOT NULL,
    team_id uuid REFERENCES teams (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    tag     varchar,
    steamid varchar,
    PRIMARY KEY (id)
);

create table servers
(
    id       uuid NOT NULL DEFAULT uuid_generate_v4(),
    host     inet NOT NULL,
    port     int           DEFAULT 270.15 NOT NULL
        CONSTRAINT valid_port CHECK ( port >= 1 AND port <= 65535 ),
    type     varchar,
    password text NOT NULL DEFAULT '',
    PRIMARY KEY (id)
);

CREATE TABLE spectators
(
    id      uuid    NOT NULL DEFAULT uuid_generate_v4(),
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
    id                  uuid                                                             NOT NULL DEFAULT uuid_generate_v4(),
    server_id           uuid REFERENCES servers (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    team1_id            uuid REFERENCES teams (id) ON UPDATE CASCADE ON DELETE CASCADE   NOT NULL,
    team2_id            uuid REFERENCES teams (id) ON UPDATE CASCADE ON DELETE CASCADE   NOT NULL,
    team1_score         integer,
    team2_score         integer,
    num_maps            integer                                                          NOT NULL,
    skip_veto           bool                                                             NOT NULL,
    veto_first          side                                                             NOT NULL,
    players_per_team    integer                                                                   DEFAULT 5 CHECK ( players_per_team > 0 OR players_per_team IS NULL ) NOT NULL,
    min_player_to_ready integer CHECK ( min_player_to_ready <= players_per_team )        NOT NULL,
    CHECK ( min_player_to_ready > 0 OR min_player_to_ready IS NULL ),
    PRIMARY KEY (id)
);

/* ordered list of maps to be played or voted on in a match */
CREATE TABLE maplist
(
    id       uuid    NOT NULL DEFAULT uuid_generate_v4(),
    match_id uuid REFERENCES matches (id) ON UPDATE CASCADE ON DELETE CASCADE,
    "order"  integer NOT NULL,
    map      varchar NOT NULL, /* people could add any map to there server, and because of this it's not possible to use an enum */
    PRIMARY KEY (id),
    UNIQUE (id, "order")
);

CREATE TABLE match_spectator
(
    match_id     uuid REFERENCES matches (id) ON UPDATE CASCADE ON DELETE CASCADE    NOT NULL,
    spectator_id uuid REFERENCES spectators (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    PRIMARY KEY (match_id, spectator_id)
);