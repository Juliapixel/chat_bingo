CREATE TYPE bingo_item AS (
    inner_text text,
    picked boolean
);

CREATE TABLE games (
    id serial PRIMARY KEY,
    creator_id text NOT NULL,
    creation_date timestamp with time zone NOT NULL,
    board_size integer NOT NULL,
    items bingo_item[] NOT NULL
);

CREATE TABLE users (
    id serial PRIMARY KEY,
    user_id uuid,
    twitch_id text NOT NULL UNIQUE,
    twitch_login text NOT NULL UNIQUE,
    twitch_token text
);

CREATE TABLE players (
    id serial PRIMARY KEY,
    user_id integer NOT NULL REFERENCES users (id),
    game_id integer NOT NULL REFERENCES games (id),
    items integer[]
);

CREATE INDEX players_twitch_id_game_id on players (user_id, game_id);
