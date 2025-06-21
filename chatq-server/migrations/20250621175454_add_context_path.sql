-- Add migration script here
CREATE EXTENSION IF NOT EXISTS ltree;

ALTER TABLE messages
    ADD COLUMN context_path ltree;

UPDATE messages SET context_path = text2ltree(context)
WHERE context IS NOT NULL;

ALTER TABLE messages
    ALTER COLUMN context_path SET NOT NULL;

CREATE INDEX messages_context_path_gist ON messages USING GIST (context_path);