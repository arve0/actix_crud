create table user (
    username text primary key not null,
    password text not null
);
create table user_session (
  key blob primary key not null,
  username text not null
);
