-- One row per short link. The code is the primary key, so the database itself
-- guarantees that two links can never share a code.
CREATE TABLE links (
    code       TEXT    PRIMARY KEY,
    url        TEXT    NOT NULL,
    hits       INTEGER NOT NULL DEFAULT 0 CHECK (hits >= 0),
    created_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
