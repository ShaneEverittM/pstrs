DROP TABLE IF EXISTS pastes;

CREATE TABLE pastes
(
    id      uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    content TEXT NOT NULL
);