CREATE TABLE user_skills (
    user_id INT NOT NULL REFERENCES users(user_id),
    skill_id INT NOT NULL REFERENCES skills(skill_id),
    PRIMARY KEY (user_id, skill_id)
);