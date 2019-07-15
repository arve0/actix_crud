create table users (
    username text primary key not null,
    password text not null
);
create table user_sessions (
  key blob primary key not null,
  username text not null
);
