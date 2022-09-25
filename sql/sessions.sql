CREATE TABLE sessions (
    id integer NOT NULL PRIMARY KEY,
    account_id integer NOT NULL,
    character_id integer NOT NULL,
    world_id smallint NOT NULL,
    channel_id smallint NOT NULL,
);

CREATE UNIQUE INDEX sessions_pkey ON session(id int4_ops);