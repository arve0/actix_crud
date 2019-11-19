CREATE TABLE document (
    id text not null,
    username text not null,
    data text not null,
    primary key (id, username)
);
CREATE TABLE user (
    username text primary key not null,
    password text not null
);
CREATE TABLE user_session (
  key blob primary key not null,
  username text not null
);
CREATE TABLE setting (
    name text primary key,
    value blob
);
