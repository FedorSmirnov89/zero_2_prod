use actix_web::{
    http::header::ContentType,
    web::{self, ReqData},
    HttpResponse,
};
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{authentication::UserId, utils::e500};

pub async fn admin_dashboard(
    user_id: ReqData<UserId>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user_id.into_inner();
    let username = get_username(*user_id, &pool).await.map_err(e500)?;
    let response = HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"
            <!DOCTYPE html>
<html lang="en">

<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Admin Dashboard</title>
</head>

<body>
    <p>Welcome, {username}!</p>

    <p>Available actions:</p>
    <ol>
        <li><a href="/admin/password">Change password</a></li>
        <li>
            <form name="logoutForm" action="/admin/logout" method="post">
                <input type="submit" value="Logout">
            </form>
        </li>
    </ol>
</body>

</html>
"#
        ));

    Ok(response)
}

#[tracing::instrument(name = "get username", skip(pool))]
pub async fn get_username(user_id: Uuid, pool: &PgPool) -> Result<String, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        SELECT username
        FROM users
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await
    .context("failed to perform a query to retrieve a username")?;
    Ok(row.username)
}
