
// 创建数据库和表

create database newsletter

CREATE TABLE subscriptions(
id uuid NOT NULL,
PRIMARY KEY (id),
email TEXT NOT NULL UNIQUE,
name TEXT NOT NULL,
subscribed_at timestamptz NOT NULL
);

CREATE TABLE subscription_tokens(
    subscription_token TEXT NOT NULL,
    subscriber_id      uuid NOT NULL
        REFERENCES subscriptions (id),
    PRIMARY KEY (subscription_token)
);
CREATE TABLE users(
     user_id uuid PRIMARY KEY,
     username TEXT NOT NULL UNIQUE,
     password_hash TEXT NOT NULL
);
