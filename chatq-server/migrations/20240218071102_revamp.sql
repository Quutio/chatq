-- Add migration script here
CREATE EXTENSION IF NOT EXISTS ltree;

CREATE TABLE audiences
(
    id bigserial PRIMARY KEY,
    users uuid[] UNIQUE NOT NULL,
    users_hash TEXT NOT NULL
);

CREATE TABLE sources
(
    id   bigserial PRIMARY KEY,
    uuid uuid UNIQUE NOT NULL
);

CREATE TABLE source_audiences
(
    id          bigserial PRIMARY KEY,
    source_id   bigint NOT NULL REFERENCES sources (id),
    audience_id bigint NOT NULL REFERENCES audiences (id),
    UNIQUE (source_id, audience_id)
);

CREATE TABLE messages
(
    id          BIGSERIAL PRIMARY KEY,
    issued      TIMESTAMP NOT NULL,
    content     TEXT      NOT NULL,
    context     TEXT      NOT NULL,
    audience_id bigint NOT NULL REFERENCES audiences (id),
    source_id   bigint NOT NULL REFERENCES sources (id)
);

CREATE TABLE query_snapshots
(
    id uuid PRIMARY KEY,
    query_json TEXT NOT NULL,
    snapshot_taken TIMESTAMP NOT NULL,
    target bigint NOT NULL REFERENCES sources(id)
);

CREATE TABLE message_snapshots
(
    snapshot_id uuid NOT NULL REFERENCES query_snapshots(id),
    message_id BIGINT NOT NULL REFERENCES messages(id),
    PRIMARY KEY (snapshot_id, message_id)
);