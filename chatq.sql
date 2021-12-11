CREATE TABLE messages
(
    id BIGSERIAL NOT NULL,
    ts TIMESTAMP NOT NULL,
    content TEXT NOT NULL,
    PRIMARY KEY(id)
);

CREATE TABLE audiences_server
(
    id BIGSERIAL NOT NULL,
    server VARCHAR(64) NOT NULL,
    PRIMARY KEY(server),
    CONSTRAINT fk_id
        FOREIGN KEY(id)
            REFERENCES messages(id)
);

CREATE TABLE audiences_player
(
    id BIGSERIAL NOT NULL,
    uuid UUID NOT NULL,
    PRIMARY KEY(uuid),
    CONSTRAINT fk_id
        FOREIGN KEY(id)
            REFERENCES messages(id)
);

CREATE TABLE sources_plugin
(
    id BIGSERIAL NOT NULL,
    uuid UUID NOT NULL,
    PRIMARY KEY(uuid),
    CONSTRAINT fk_id
        FOREIGN KEY(id)
            REFERENCES messages(id)
);

CREATE TABLE sources_player
(
    id BIGSERIAL NOT NULL,
    uuid UUID NOT NULL,
    PRIMARY KEY(uuid),
    CONSTRAINT fk_id
        FOREIGN KEY(id)
            REFERENCES messages(id)
);
