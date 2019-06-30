update entries set
    id=:id,
    revision=:revision,
    hash=:hash,
    prev_hash=:prev_hash,
    data=:data
where
    id=:id;