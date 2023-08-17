use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),  // tells tracing to skip these arguments (default is to attach all function arguments to the context of the span)
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscriptions(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    match insert_subscriber(&pool, &form).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            error!("failed to executed query: {e:?}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[tracing::instrument(
    name = "saving new subscriber details in the database",
    skip(form, pool)
)]
async fn insert_subscriber(pool: &PgPool, form: &FormData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at)
            VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        error!("failed to execute query {e:?}");
        e
    })?;
    Ok(())
}

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}
