select
    id
    ,revision
    ,hash
    ,prev_hash
    ,data
from entries
where id=?1
;