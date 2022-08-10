-- Table Definition ----------------------------------------------

CREATE TABLE cooldowns (
    id integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    character_id integer NOT NULL,
    skill_id integer NOT NULL,
    length bigint NOT NULL,
    start_time bigint NOT NULL
);

-- Indices -------------------------------------------------------

CREATE UNIQUE INDEX cooldowns_pkey ON cooldowns(id int4_ops);