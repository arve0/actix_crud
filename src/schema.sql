CREATE TABLE sakkosekk(
    id          INTEGER NOT NULL
    ,revision    INTEGER NOT NULL
    ,hash        BLOB NOT NULL
    ,prev_hash   BLOB NOT NULL
    ,json        TEXT NOT NULL
)