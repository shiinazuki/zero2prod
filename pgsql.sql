
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

CREATE TYPE header_pair AS (
name TEXT,
value BYTEA
);

CREATE TABLE idempotency (
user_id uuid NOT NULL REFERENCES users(user_id),
idempotency_key TEXT NOT NULL,
response_status_code SMALLINT NOT NULL,
response_headers header_pair[] NOT NULL,
response_body BYTEA NOT NULL,
created_at timestamptz NOT NULL,
PRIMARY KEY(user_id, idempotency_key)
);


CREATE TABLE newsletter_issues (
newsletter_issue_id uuid NOT NULL,
title TEXT NOT NULL,
text_content TEXT NOT NULL,
html_content TEXT NOT NULL,
published_at TEXT NOT NULL,
PRIMARY KEY(newsletter_issue_id)
);

CREATE TABLE issue_delivery_queue (
newsletter_issue_id uuid NOT NULL REFERENCES newsletter_issues (newsletter_issue_id),
subscriber_email TEXT NOT NULL,
PRIMARY KEY(newsletter_issue_id, subscriber_email)
);
