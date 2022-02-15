create table accounts
(
    id              serial
        constraint accounts_pk
            primary key,
    name            varchar(13)  default ''::character varying                              not null,
    password        varchar(128) default ''::character varying                              not null,
    pin             varchar(10)  default ''::character varying                              not null,
    pic             varchar(26)  default ''::character varying                              not null,
    login_state     smallint     default 0                                                  not null,
    last_login      timestamp,
    create_date     timestamp    default CURRENT_TIMESTAMP                                  not null,
    birthday        date         default '2015-01-01'::date                                 not null,
    banned          boolean      default false                                              not null,
    ban_reason      text,
    macs            text,
    nx_credit       integer,
    maple_points    integer,
    nx_prepaid      integer,
    character_slots smallint     default 3                                                  not null,
    gender          smallint     default 0                                                  not null,
    temp_ban        timestamp    default '2015-01-01 05:00:00'::timestamp without time zone not null,
    greason         smallint     default 0                                                  not null,
    accepted_tos    boolean      default false                                              not null,
    site_logged     text,
    web_admin       boolean      default false,
    nick            varchar(20)  default NULL::character varying,
    mute            boolean      default false,
    email           varchar(45)  default NULL::character varying,
    ip              text,
    reward_points   integer      default 0                                                  not null,
    vote_points     integer      default 0                                                  not null,
    hwid            varchar(12)  default ''::character varying                              not null,
    language        smallint     default 2                                                  not null,
    constraint ranking1
        unique (id, banned)
);

create index accounts_id_name_index
    on accounts (id, name);

create index accounts_id_nx_credit_maple_points_nx_prepaid_index
    on accounts (id, nx_credit, maple_points, nx_prepaid);

create unique index accounts_name_uindex
    on accounts (name);
