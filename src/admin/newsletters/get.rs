use actix_web::{http::header::ContentType, HttpResponse};

pub async fn newsletter_form() -> Result<HttpResponse, actix_web::Error> {
    let response = HttpResponse::Ok().content_type(ContentType::html()).body(
        r#"
            <!DOCTYPE html>
<html lang="en">

<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Admin Dashboard</title>
</head>

<body>
    <p>Enter the newsletter issue below!</p>

    <form name="newsletterForm" action="/admin/newsletters" method="post">
        <label>Newsletter Title
            <input type="text" placeholder="enter the newsletter title" name="title">
        </label>
        <label>Newsletter Content
            <input type="text" placeholder="enter the newsletter content" name="content">
        </label>
        <br>
        <button type="submit">Send Newsletter</button>
    </form>
</body>

</html>
"#,
    );
    Ok(response)
}
