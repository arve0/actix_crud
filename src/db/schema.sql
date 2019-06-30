create table entries (
    id text primary key not null
    ,revision integer not null
    ,hash blob not null
    ,prev_hash blob
    ,data text not null
);