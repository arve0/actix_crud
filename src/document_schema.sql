create table document (
    id text not null,
    created number not null,
    updated number not null,
    username text not null,
    data text not null,
    primary key (id, username)
);
