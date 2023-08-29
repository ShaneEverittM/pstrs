DROP TABLE IF EXISTS pastes;

CREATE TABLE pastes
(
    id      serial PRIMARY KEY,
    content TEXT NOT NULL
);