CREATE TABLE messages
(
    message_id BIGSERIAL NOT NULL,
    issued TIMESTAMP NOT NULL,
    content TEXT NOT NULL,
    PRIMARY KEY(message_id)
);

CREATE TABLE audiences_server
(
    id BIGSERIAL NOT NULL,
    server VARCHAR(64) NOT NULL,
    message_id BIGINT NOT NULL,
    PRIMARY KEY(id),
    CONSTRAINT fk__audiences_server__messages
        FOREIGN KEY(message_id)
            REFERENCES messages(message_id)
);

CREATE TABLE audiences_player
(
    id BIGSERIAL NOT NULL,
    player UUID NOT NULL,
    message_id BIGINT NOT NULL,
    PRIMARY KEY(id),
    CONSTRAINT fk__audiences_player__messages
        FOREIGN KEY(message_id)
            REFERENCES messages(message_id)
);

CREATE TABLE sources_plugin
(
    id BIGSERIAL NOT NULL,
    plugin VARCHAR(64) NOT NULL,
    message_id BIGINT NOT NULL,
    PRIMARY KEY(id),
    CONSTRAINT fk__sources_plugin__messages
        FOREIGN KEY(message_id)
            REFERENCES messages(message_id)
);

CREATE TABLE sources_player
(
    id BIGSERIAL NOT NULL,
    player UUID NOT NULL,
    message_id BIGINT NOT NULL,
    PRIMARY KEY(id),
    CONSTRAINT fk__sources_player__messages
        FOREIGN KEY(message_id)
            REFERENCES messages(message_id)
);
