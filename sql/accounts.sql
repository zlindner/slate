-- Table Definition ----------------------------------------------

CREATE TABLE accounts (
    id SERIAL PRIMARY KEY,
    name character varying(13) NOT NULL DEFAULT ''::character varying,
    password character varying(128) NOT NULL DEFAULT ''::character varying,
    pin character varying(10) NOT NULL DEFAULT ''::character varying,
    pic character varying(26) NOT NULL DEFAULT ''::character varying,
    login_state smallint NOT NULL DEFAULT 0,
    last_login timestamp without time zone,
    create_date timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    birthday date NOT NULL DEFAULT '2015-01-01'::date,
    banned boolean NOT NULL DEFAULT false,
    ban_reason text,
    macs text,
    nx_credit integer,
    maple_points integer,
    nx_prepaid integer,
    character_slots smallint NOT NULL DEFAULT 3,
    gender smallint NOT NULL DEFAULT 0,
    temp_ban timestamp without time zone NOT NULL DEFAULT '2015-01-01 05:00:00'::timestamp without time zone,
    greason smallint NOT NULL DEFAULT 0,
    accepted_tos boolean NOT NULL DEFAULT false,
    site_logged text,
    web_admin boolean DEFAULT false,
    nick character varying(20) DEFAULT NULL::character varying,
    mute boolean DEFAULT false,
    email character varying(45) DEFAULT NULL::character varying,
    ip text,
    reward_points integer NOT NULL DEFAULT 0,
    vote_points integer NOT NULL DEFAULT 0,
    hwid character varying(12) NOT NULL DEFAULT ''::character varying,
    language smallint NOT NULL DEFAULT 2,
    CONSTRAINT accounts_ranking1 UNIQUE (id, banned)
);

-- Indices -------------------------------------------------------

CREATE UNIQUE INDEX accounts_pk ON accounts(id int4_ops);
CREATE UNIQUE INDEX accounts_ranking1 ON accounts(id int4_ops,banned bool_ops);
CREATE INDEX accounts_id_name_index ON accounts(id int4_ops,name text_ops);
CREATE INDEX accounts_id_nx_credit_maple_points_nx_prepaid_index ON accounts(id int4_ops,nx_credit int4_ops,maple_points int4_ops,nx_prepaid int4_ops);
CREATE UNIQUE INDEX accounts_name_uindex ON accounts(name text_ops);
