CREATE TABLE users (
    id          BLOB NOT NULL UNIQUE PRIMARY KEY, -- UUIDv7 as bytes
    handle      TEXT NOT NULL UNIQUE,
    password    TEXT, -- hashed, nullable in case of OAuth2-only login
    prof_pic    TEXT  -- link probably
);
CREATE TABLE sessions (
    id          BLOB NOT NULL UNIQUE PRIMARY KEY, -- UUIDv7 as bytes
    token       BLOB NOT NULL UNIQUE,
    user_id     BLOB NOT NULL REFERENCES users(id), -- UUIDv7 bytes (userID)
    expiry      TEXT NOT NULL, -- RFC3339 into DateTime<Utc>
    revoked     INTEGER NOT NULL DEFAULT 0, -- bool (int 0 or int 1)
    revoked_at  TEXT DEFAULT NULL, -- RFC3339 into DateTime<Utc>
    revoked_by  BLOB DEFAULT NULL REFERENCES users(id)  -- UUIDv7 bytes (userID)

    CHECK(
        (revoked = 0 AND revoked_at IS NULL AND revoked_by IS NULL) OR
        (revoked = 1 AND revoked_at IS NOT NULL AND revoked_by IS NOT NULL)
    )
);
CREATE INDEX sessions_by_userid ON sessions(user_id);

-- CREATE TABLE logs (
--     id          BLOB NOT NULL UNIQUE PRIMARY KEY, -- UUIDv7 as bytes
--     actor       BLOB NOT NULL REFERENCES users(id), -- UUIDv7 as bytes
--     -- (userID with special cases: UUID::nil if system, UUID::max if infradmin)
--     -- ((infradmin & system shall both be users))
--     target      BLOB, -- Option<UUIDv7 as bytes (userID)>
--     change      TEXT NOT NULL
-- );

CREATE TABLE quotes (
    id          BLOB NOT NULL UNIQUE PRIMARY KEY, -- UUIDv7 as bytes
    timestamp   TEXT NOT NULL, -- RFC3339 into DateTime<Utc>
    location    TEXT,
    context     TEXT,
    created_by  BLOB NOT NULL REFERENCES users(id), -- UUIDv7 as bytes
    public      INTEGER NOT NULL DEFAULT 0 -- bool (int 0 or int 1)
    -- this is to be followed by a bigger role-based viewership scoping mechanism
);
CREATE INDEX quotes_by_creation_user ON quotes(created_by);
CREATE TABLE persons (
    id          BLOB NOT NULL UNIQUE PRIMARY KEY, -- UUIDv7 as bytes
    created_by  BLOB NOT NULL REFERENCES users(id), -- UUIDv7 as bytes
    bio         TEXT,
    prof_pic    TEXT -- link probably
);
CREATE TABLE names (
    id          BLOB NOT NULL UNIQUE PRIMARY KEY, -- UUIDv7 as bytes
    is_primary  INTEGER NOT NULL DEFAULT 0,
    person_id   BLOB NOT NULL REFERENCES persons(id),
    created_by  BLOB NOT NULL REFERENCES users(id),
    name        TEXT NOT NULL
);
CREATE INDEX names_by_personid ON names(person_id);
CREATE UNIQUE INDEX no_name_duplicate_for_same_person ON names(person_id, name);
CREATE UNIQUE INDEX primary_name_uniqueness ON names(person_id) WHERE is_primary = 1;
CREATE TABLE lines (
    id          BLOB NOT NULL UNIQUE PRIMARY KEY, -- UUIDv7 as bytes
    quote_id    BLOB NOT NULL REFERENCES quotes(id), -- UUIDv7 as bytes
    name_id     BLOB NOT NULL REFERENCES names(id), -- UUIDv7 as bytes
    ordering    INTEGER NOT NULL,
    content     TEXT NOT NULL
);
CREATE INDEX lines_by_quoteid ON lines(quote_id);
CREATE INDEX lines_by_nameid ON lines(name_id);
CREATE UNIQUE INDEX lines_unique_ordering ON lines(quote_id, ordering);

CREATE TABLE tags (
    id          BLOB NOT NULL UNIQUE PRIMARY KEY, -- UUIDv7 as bytes
    tagname     TEXT NOT NULL UNIQUE
);

CREATE TABLE user_quote_likes (
    quote_id    BLOB NOT NULL REFERENCES quotes(id), -- UUIDv7 as bytes
    user_id     BLOB NOT NULL REFERENCES users(id), -- UUIDv7 as bytes
    PRIMARY KEY (quote_id, user_id)
) WITHOUT ROWID;
CREATE INDEX likes_by_reverse_index ON user_quote_likes(user_id, quote_id);

CREATE TABLE quote_tags (
    quote_id    BLOB NOT NULL REFERENCES quotes(id), -- UUIDv7 as bytes
    tag_id      BLOB NOT NULL REFERENCES tags(id), -- UUIDv7 as bytes
    PRIMARY KEY (quote_id, tag_id)
) WITHOUT ROWID;
CREATE INDEX quote_tags_reverse_index ON quote_tags(tag_id, quote_id);

-- all this to be followed by:
-- - a better access scoping mechanism (role-based like discord)
-- - photos just like quotes
-- - OAuth2 login via Steam/GitHub/Discord/Google/Potato/Whatever
-- - comments
