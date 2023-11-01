use actix_web::{
    web::{self, ReqData},
    HttpResponse,
};
use actix_web_flash_messages::FlashMessage;
use anyhow::{Context, Result};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    authentication::UserId,
    idempotency::{save_response, try_processing, IdempotencyKey, NextAction},
    utils::{e400, e500, see_other},
};

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: String,
    idempotency_key: String,
}

#[tracing::instrument(skip_all)]
async fn enqueue_delivery_tasks(
    transaction: &mut Transaction<'_, Postgres>,
    newsletter_issue_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO issue_delivery_queue(
            newsletter_issue_id,
            subscriber_email
        )
        SELECT $1, email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
        newsletter_issue_id
    )
    .execute(transaction)
    .await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
async fn insert_newsletter_issue(
    transaction: &mut Transaction<'_, Postgres>,
    title: &str,
    content: &str,
) -> Result<Uuid, sqlx::Error> {
    let newsletter_issue_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO newsletter_issues(
            newsletter_issue_id,
            title,
            content,
            published_at
        )
        VALUES($1, $2, $3, now())
        "#,
        newsletter_issue_id,
        title,
        content
    )
    .execute(transaction)
    .await?;
    Ok(newsletter_issue_id)
}

#[tracing::instrument(
    name = "publish a newsletter issue",
    skip(body, pool, user_id),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn publish_newsletter(
    user_id: ReqData<UserId>,
    body: web::Form<BodyData>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let BodyData {
        title,
        content,
        idempotency_key,
    } = body.into_inner();
    let idempotency_key: IdempotencyKey = idempotency_key.try_into().map_err(e400)?;
    let user_id = *user_id.into_inner();
    let success_message = FlashMessage::info("The newsletter issue has been published.");

    let next_action = try_processing(&pool, &idempotency_key, user_id)
        .await
        .map_err(e500)?;

    let mut transaction = match next_action {
        NextAction::StartProcessing(t) => t,
        NextAction::ReturnSavedResponse(saved_response) => {
            success_message.send();
            return Ok(saved_response);
        }
    };

    let issue_id = insert_newsletter_issue(&mut transaction, &title, &content)
        .await
        .context("failed to store newsletter issue details")
        .map_err(e500)?;

    enqueue_delivery_tasks(&mut transaction, issue_id)
        .await
        .context("failed to enqueue delivery tasks")
        .map_err(e500)?;
    success_message.send();
    let response = see_other("/admin/newsletters");
    let response = save_response(&idempotency_key, user_id, response, transaction)
        .await
        .map_err(e500)?;
    Ok(response)
}
