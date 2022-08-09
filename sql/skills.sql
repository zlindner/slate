-- Table Definition ----------------------------------------------

CREATE TABLE skills (
    id integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    skill_id integer NOT NULL DEFAULT 0,
    character_id integer NOT NULL DEFAULT 0,
    level integer NOT NULL DEFAULT 0,
    mastery_level integer NOT NULL DEFAULT 0,
    expiration bigint NOT NULL DEFAULT '-1'::bigint
);

-- Indices -------------------------------------------------------

CREATE UNIQUE INDEX skills_pkey ON skills(id int4_ops);
CREATE INDEX skill_pair ON skills(skill_id int4_ops,character_id int4_ops);
