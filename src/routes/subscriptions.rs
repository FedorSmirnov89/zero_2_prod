use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

use crate::domain::{
    new_subscriber::NewSubscriber, subscriber_email::SubscriberEmail,
    subscriber_name::SubscriberName,
};

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),  // tells tracing to skip these arguments (default is to attach all function arguments to the context of the span)
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let new_subscriber = match form.0.try_into() {
        Ok(s) => s,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    match insert_subscriber(&pool, &new_subscriber).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            error!("failed to executed query: {e:?}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(form: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(form.name)?;
        let email = SubscriberEmail::parse(form.email)?;
        Ok(NewSubscriber { name, email })
    }
}

#[tracing::instrument(
    name = "saving new subscriber details in the database",
    skip(new_subscriber, pool)
)]
async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at, status)
            VALUES ($1, $2, $3, $4, 'confirmed')
        "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
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
