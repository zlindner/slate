-- auto-generated definition
create table characters
(
    id                     serial
        constraint characters_pk
            primary key,
    account_id             integer      default 0                                                  not null
        constraint characters_account_id
            unique,
    world                  integer      default 0                                                  not null,
    name                   varchar(13)  default ''::character varying                              not null,
    level                  integer      default 1                                                  not null,
    exp                    integer      default 0                                                  not null,
    gacha_exp              integer      default 0                                                  not null,
    str                    integer      default 12                                                 not null,
    dex                    integer      default 5                                                  not null,
    luk                    integer      default 4                                                  not null,
    int                    integer      default 4                                                  not null,
    hp                     integer      default 50                                                 not null,
    mp                     integer      default 5                                                  not null,
    max_hp                 integer      default 50                                                 not null,
    max_mp                 integer      default 5                                                  not null,
    mesos                  integer      default 0                                                  not null,
    hp_mp_used             integer      default 0                                                  not null,
    job                    integer      default 0                                                  not null,
    skin_color             integer      default 0                                                  not null,
    gender                 smallint     default 0                                                  not null,
    fame                   integer      default 0                                                  not null,
    fquest                 integer      default 0                                                  not null,
    hair                   integer      default 0                                                  not null,
    face                   integer      default 0                                                  not null,
    ap                     integer      default 0                                                  not null,
    sp                     varchar(128) default '0,0,0,0,0,0,0,0,0,0'::character varying           not null,
    map                    integer      default 0                                                  not null,
    spawn_point            integer      default 0                                                  not null,
    gm                     smallint     default 0                                                  not null,
    party                  integer      default 0                                                  not null
        constraint characters_party
            unique,
    buddy_capacity         integer      default 25                                                 not null,
    create_date            timestamp    default CURRENT_TIMESTAMP                                  not null,
    rank                   integer      default 1                                                  not null,
    rank_move              integer      default 0                                                  not null,
    job_rank               integer      default 1                                                  not null,
    job_rank_move          integer      default 0                                                  not null,
    guild_id               integer      default 0                                                  not null,
    guild_rank             integer      default 5                                                  not null,
    messenger_id           integer      default 0                                                  not null,
    messenger_position     integer      default 4                                                  not null,
    mount_level            integer      default 1                                                  not null,
    mount_exp              integer      default 0                                                  not null,
    mount_tiredness        integer      default 0                                                  not null,
    omok_wins              integer      default 0                                                  not null,
    omok_losses            integer      default 0                                                  not null,
    omok_ties              integer      default 0                                                  not null,
    match_card_wins        integer      default 0                                                  not null,
    match_card_losses      integer      default 0                                                  not null,
    match_card_ties        integer      default 0                                                  not null,
    merchant_mesos         integer      default 0,
    has_merchant           boolean      default false,
    equip_slots            integer      default 24                                                 not null,
    use_slots              integer      default 24                                                 not null,
    setup_slots            integer      default 24                                                 not null,
    etc_slots              integer      default 24                                                 not null,
    family_id              integer      default '-1'::integer                                      not null,
    monster_book_cover     integer      default 0                                                  not null,
    alliance_rank          integer      default 5                                                  not null,
    vanquisher_stage       integer      default 0                                                  not null,
    ariant_points          integer      default 0                                                  not null,
    dojo_points            integer      default 0                                                  not null,
    last_dojo_stage        integer      default 0                                                  not null,
    finished_dojo_tutorial boolean      default false                                              not null,
    vanquisher_kills       integer      default 0                                                  not null,
    summon_value           integer      default 0                                                  not null,
    partner_id             integer      default 0                                                  not null,
    marriage_item_id       integer      default 0                                                  not null,
    rebirths               integer      default 0                                                  not null,
    pq_points              integer      default 0                                                  not null,
    data_string            varchar(64)  default ''::character varying                              not null,
    last_logout_time       timestamp    default '2015-01-01 05:00:00'::timestamp without time zone not null,
    last_exp_gain_time     timestamp    default '2015-01-01 05:00:00'::timestamp without time zone not null,
    party_search           boolean      default true                                               not null,
    jail_expire            integer      default 0                                                  not null,
    constraint characters_ranking1
        unique (level, exp),
    constraint characters_ranking2
        unique (gm, job)
);

create index characters_id_account_id_name_index
    on characters (id, account_id, name);

create index characters_id_account_id_world_index
    on characters (id, account_id, world);

