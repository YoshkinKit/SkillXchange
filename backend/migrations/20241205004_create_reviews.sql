CREATE TABLE reviews (
    review_id SERIAL PRIMARY KEY,
    sender_id INT NOT NULL REFERENCES users(user_id),
    receiver_id INT NOT NULL REFERENCES users(user_id),
    skill_id INT REFERENCES skills(skill_id),
    rating INT CHECK (rating BETWEEN 0 AND 5),
    comment TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);