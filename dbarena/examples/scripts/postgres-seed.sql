-- PostgreSQL Seed Data Example
-- Inserts sample data for testing

-- Insert sample users
INSERT INTO users (username, email) VALUES
    ('admin', 'admin@example.com'),
    ('john_doe', 'john@example.com'),
    ('jane_smith', 'jane@example.com'),
    ('bob_wilson', 'bob@example.com')
ON CONFLICT (username) DO NOTHING;

-- Insert sample posts
INSERT INTO posts (user_id, title, content, published) VALUES
    (1, 'Welcome to the Blog', 'This is the first post on our new blog!', TRUE),
    (1, 'Database Setup Guide', 'Learn how to set up your database...', TRUE),
    (2, 'My First Post', 'Hello everyone, this is my first post.', TRUE),
    (3, 'Learning PostgreSQL', 'PostgreSQL is a powerful database...', TRUE),
    (4, 'Draft Post', 'This is a draft post.', FALSE);

-- Insert sample comments
INSERT INTO comments (post_id, user_id, content) VALUES
    (1, 2, 'Great post! Thanks for sharing.'),
    (1, 3, 'Very helpful, looking forward to more.'),
    (2, 4, 'Excellent guide, saved me a lot of time.'),
    (3, 1, 'Welcome to the platform!'),
    (3, 4, 'Nice to meet you!'),
    (4, 2, 'PostgreSQL is awesome!');

-- Print success message
SELECT
    (SELECT COUNT(*) FROM users) AS users_count,
    (SELECT COUNT(*) FROM posts) AS posts_count,
    (SELECT COUNT(*) FROM comments) AS comments_count;
