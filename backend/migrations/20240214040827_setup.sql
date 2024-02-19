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
    user_id uuid NOT NULL,
    twitch_id text NOT NULL UNIQUE,
    twitch_login text NOT NULL UNIQUE,
    twitch_display_name text NOT NULL UNIQUE
);

CREATE TABLE twitch_tokens (
    id serial PRIMARY KEY,
    user_id integer NOT NULL UNIQUE REFERENCES users (id) ON DELETE CASCADE,
    token text NOT NULL,
    issued_at timestamp with time zone NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    refresh_token text NOT NULL
);

CREATE INDEX expires_at ON twitch_tokens (expires_at);

CREATE TABLE players (
    id serial PRIMARY KEY,
    user_id integer NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    game_id integer NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    items integer[]
);

-- CREATE INDEX players_twitch_id_game_id on players (user_id, game_id);
