use crate::authentication::UserId;
use crate::domain::SubscriberEmail;
use crate::idempotency::IdempotencyKey::IdempotencyKey;
use crate::idempotency::{ save_response, try_processing, NextAction};
use crate::utils::{e400, e500, see_other};
use actix_web::web::ReqData;
use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use sqlx::{PgPool, Postgres, Transaction};
use std::ops::DerefMut;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    title: String,
    html_content: String,
    message_stream: String,
    idempotency_key: String,
}

#[tracing::instrument(skip_all)]
async fn enqueue_delivery_tasks(
    transaction: &mut Transaction<'_, Postgres>,
    newsletter_issue_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
INSERT INTO issue_delivery_queue (
newsletter_issue_id,
subscriber_email
)
SELECT $1, email
FROM subscriptions
WHERE status = 'confirmed'
"#,
        newsletter_issue_id,
    )
    .execute(transaction.deref_mut())
    .await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn insert_newsletter_issue(
    transaction: &mut Transaction<'_, Postgres>,
    title: &str,
    message_stream: &str,
    html_content: &str,
) -> Result<Uuid, sqlx::Error> {
    let newsletter_issue_id = Uuid::new_v4();
    sqlx::query!(
        r#"
INSERT INTO newsletter_issues (
newsletter_issue_id,
title,
text_content,
html_content,
published_at
)
VALUES ($1, $2, $3, $4, now())
"#,
        newsletter_issue_id,
        title,
        message_stream,
        html_content
    )
    .execute(transaction.deref_mut())
    .await?;
    Ok(newsletter_issue_id)
}

#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip_all,
    fields(user_id=%*user_id)
)]
pub async fn publish_newsletter(
    form: web::Form<FormData>,
    user_id: ReqData<UserId>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user_id.into_inner();
    let FormData {
        title,
        html_content,
        message_stream,
        idempotency_key,
    } = form.0;
    let idempotency_key: IdempotencyKey = idempotency_key.try_into().map_err(e400)?;
    let mut transaction = match try_processing(&pool, &idempotency_key, *user_id)
        .await
        .map_err(e500)?
    {
        NextAction::StartProcessing(t) => t,
        NextAction::ReturnSaveResponse(saved_response) => {
            success_message().send();
            return Ok(saved_response);
        }
    };
    let issue_id =
        insert_newsletter_issue(&mut transaction, &title, &message_stream, &html_content)
            .await
            .context("Failed to store newsletter issue details")
            .map_err(e500)?;

    enqueue_delivery_tasks(&mut transaction, issue_id)
        .await
        .context("Failed to enqueue delivery tasks")
        .map_err(e500)?;

    let response = see_other("/admin/newsletters");
    let response = save_response(transaction, &idempotency_key, *user_id, response)
        .await
        .map_err(e500)?;

    success_message().send();

    Ok(response)
}

fn success_message() -> FlashMessage {
    FlashMessage::info("The newsletter issue has been accepted - emails will go out shortly.")
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let confirmed_subscribers = sqlx::query!(
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| match SubscriberEmail::parse(r.email) {
        Ok(email) => Ok(ConfirmedSubscriber { email }),
        Err(error) => Err(anyhow::anyhow!(error)),
    })
    .collect();
    Ok(confirmed_subscribers)
}
