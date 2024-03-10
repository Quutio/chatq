-- Add migration script here

CREATE TABLE audiences
(
    id bigserial PRIMARY KEY,
    users uuid[] NOT NULL,
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
    source_id   bigint REFERENCES sources (id),
    audience_id bigint REFERENCES audiences (id)
);

CREATE TABLE messages
(
    id          BIGSERIAL PRIMARY KEY,
    issued      TIMESTAMP NOT NULL,
    content     TEXT      NOT NULL,
    context     TEXT      NOT NULL,
    audience_id bigint REFERENCES audiences (id),
    source_id   bigint REFERENCES sources (id)
);