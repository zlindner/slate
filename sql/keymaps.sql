CREATE TABLE keymaps (
    id integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    character_id integer NOT NULL,
    key integer NOT NULL,
    type integer NOT NULL,
    action integer NOT NULL
);

CREATE UNIQUE INDEX keymaps_pkey ON keymaps(id int4_ops);