-- This script has intentional errors for testing error parsing
CREATE TABLE users (
    id INT AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(50) NOT NULL UNIQUE
);

-- Intentional syntax error (missing column type)
CREATE TABLE posts (
    id INT AUTO_INCREMENT PRIMARY KEY,
    user_id,
    title VARCHAR(200) NOT NULL
);
