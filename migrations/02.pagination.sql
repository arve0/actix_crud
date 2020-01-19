-- add columns for sorting documents
drop table document;
create table document (
    pk integer primary key,
    id text not null,
    created number not null,
    updated number not null,
    username text not null,
    data text not null
);

create unique index document_by_id_username on document(id, username);
