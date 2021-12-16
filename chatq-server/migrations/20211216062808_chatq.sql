-- Add migration script here

CREATE TABLE messages
(
    message_id BIGSERIAL NOT NULL,
    issued TIMESTAMP NOT NULL,
    content TEXT NOT NULL,
    PRIMARY KEY(message_id)
);

CREATE TABLE audiences_server
(
    server VARCHAR(64) NOT NULL,
    message_id BIGINT NOT NULL,
    PRIMARY KEY(server),
    CONSTRAINT fk__audiences_server__messages
        FOREIGN KEY(message_id)
            REFERENCES messages(message_id)
);

CREATE TABLE audiences_player
(
    player UUID NOT NULL,
    message_id BIGINT NOT NULL,
    PRIMARY KEY(player),
    CONSTRAINT fk__audiences_player__messages
        FOREIGN KEY(message_id)
            REFERENCES messages(message_id)
);

CREATE TABLE sources_plugin
(
    plugin VARCHAR(64) NOT NULL,
    message_id BIGINT NOT NULL,
    PRIMARY KEY(plugin),
    CONSTRAINT fk__sources_plugin__messages
        FOREIGN KEY(message_id)
            REFERENCES messages(message_id)
);

CREATE TABLE sources_player
(
    player UUID NOT NULL,
    message_id BIGINT,
    PRIMARY KEY(player),
    CONSTRAINT fk__sources_player__messages
        FOREIGN KEY(message_id)
            REFERENCES messages(message_id)
);
