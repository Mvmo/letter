CREATE TABLE IF NOT EXISTS badges (
    id    INTEGER PRIMARY KEY NOT NULL,
    name  TEXT                NOT NULL,
    color TEXT                NOT NULL /* ansi color format */
);

CREATE TABLE IF NOT EXISTS tasks (
    id          INTEGER PRIMARY KEY NOT NULL,
    description TEXT                NOT NULL,
    badge_id    INTEGER,

    FOREIGN KEY (badge_id) REFERENCES badges (id)
);

INSERT INTO badges (name, color) VALUES ('TODO', '2;255;157;59');
INSERT INTO badges (name, color) VALUES ('In Progress', '2;79;255;202');
INSERT INTO badges (name, color) VALUES ('Done', '2;164;255;46');
