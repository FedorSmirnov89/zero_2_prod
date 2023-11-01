use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;
use uuid::Uuid;

pub async fn newsletter_form(
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    let mut flash_msg_html = String::new();
    for m in flash_messages.iter() {
        writeln!(flash_msg_html, "<p><i>{msg}</i></p>", msg = m.content()).unwrap()
    }

    let idempotency_key = Uuid::new_v4().to_string();
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
        {flash_msg_html}
        <p>Enter the newsletter issue below!</p>
    
        <form name="newsletterForm" action="/admin/newsletters" method="post">
            <label>Newsletter Title
                <input type="text" placeholder="enter the newsletter title" name="title">
            </label>
            <label>Newsletter Content
                <input type="text" placeholder="enter the newsletter content" name="content">
            </label>
            <br>
            <input hidden type="text" name="idempotency_key" value={idempotency_key}>
            <button type="submit">Send Newsletter</button>
        </form>
    </body>
    
    </html>
    "#,
        ));
    Ok(response)
}
