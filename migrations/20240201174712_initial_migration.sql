CREATE TABLE lastfm (
    user_id INTEGER NOT NULL UNIQUE
    username VARCHAR(255)
)

CREATE TABLE routines (
    user_id INTEGER NOT NULL

    routine_type INTEGER NOT NULL
    routine_start INTEGER
    routine_end INTEGER NOT NULL

    expiry INTEGER NOT NULL
)