-- Table Definition ----------------------------------------------

CREATE TABLE skills (
    id integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    skill_id integer NOT NULL,
    character_id integer NOT NULL ,
    level integer NOT NULL,
    mastery_level integer NOT NULL,
    expiration bigint NOT NULL
);

-- Indices -------------------------------------------------------

CREATE UNIQUE INDEX skills_pkey ON skills(id int4_ops);
CREATE INDEX skill_pair ON skills(skill_id int4_ops,character_id int4_ops);
