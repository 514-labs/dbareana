-- This script has intentional errors for testing error parsing
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) NOT NULL UNIQUE
);

-- Intentional syntax error (missing comma)
CREATE TABLE posts (
    id SERIAL PRIMARY KEY
    user_id INTEGER REFERENCES users(id)
    title VARCHAR(200) NOT NULL
);
