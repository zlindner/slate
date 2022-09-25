-- Table Definition ----------------------------------------------

CREATE TABLE quests (
    id integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    character_id integer NOT NULL,
    quest_id integer NOT NULL,
    status integer NOT NULL,
    time integer NOT NULL,
    expires bigint NOT NULL,
    forfeited integer NOT NULL,
    completed integer NOT NULL,
    info smallint NOT NULL
);

-- Indices -------------------------------------------------------

CREATE UNIQUE INDEX quests_pkey ON quests(id int4_ops);
