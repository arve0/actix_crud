create table documents (
    id text not null,
    username text not null,
    data text not null,
    primary key (id, username)
);
