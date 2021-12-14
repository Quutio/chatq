-- Add migration script here

CREATE TABLE message
(
    id BIGSERIAL NOT NULL,
    ts TIMESTAMP NOT NULL,
    content TEXT NOT NULL,
    PRIMARY KEY(id)
);

CREATE TABLE audience_server
(
    id BIGSERIAL NOT NULL,
    server VARCHAR(64) NOT NULL,
    PRIMARY KEY(server),
    CONSTRAINT fk_id
        FOREIGN KEY(id)
            REFERENCES message(id)
);

CREATE TABLE audience_player
(
    id BIGSERIAL NOT NULL,
    uuid UUID NOT NULL,
    PRIMARY KEY(uuid),
    CONSTRAINT fk_id
        FOREIGN KEY(id)
            REFERENCES message(id)
);

CREATE TABLE source_plugin
(
    id BIGSERIAL NOT NULL,
    uuid UUID NOT NULL,
    PRIMARY KEY(uuid),
    CONSTRAINT fk_id
        FOREIGN KEY(id)
            REFERENCES message(id)
);

CREATE TABLE source_player
(
    id BIGSERIAL NOT NULL,
    uuid UUID NOT NULL,
    PRIMARY KEY(uuid),
    CONSTRAINT fk_id
        FOREIGN KEY(id)
            REFERENCES message(id)
);
