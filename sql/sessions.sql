CREATE TABLE sessions (
    id integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    account_id integer NOT NULL,
    character_id integer NOT NULL,
    world_id integer NOT NULL,
    channel_id smallint NOT NULL,
    login_attempts smallint NOT NULL,
    pin character varying(10) NOT NULL,
    pin_attempts smallint NOT NULL,
    pic character varying(26) NOT NULL,
    pic_attempts smallint NOT NULL
);

CREATE UNIQUE INDEX untitled_table_pkey ON session(id int4_ops);