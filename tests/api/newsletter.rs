use wiremock::{
    matchers::{any, method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::{assert_is_redirect_to, spawn_app, ConfirmationLinks, TestApp};

#[tokio::test]
async fn send_request_is_rejected_when_not_logged_in() {
    // Arrange
    let app = spawn_app().await;
    // Act
    let response = app
        .post_newsletter_issue(&serde_json::json!({
            "title": "newsletter title",
                "content": "newsletter_content"
        }))
        .await;
    // Assert
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn newsletters_returns_400_for_invalid_data() {
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        (
            serde_json::json!({
                "content": "newsletter content"
            }),
            "missing title",
        ),
        (
            serde_json::json!({
                "title": "newsletter!"
            }),
            "missing content",
        ),
    ];

    // Act I - Login
    let login_body = serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password,
    });
    let response = app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/admin/dashboard");

    // Act II - post newsletter
    for (invalid_body, error_message) in test_cases {
        let response = app.post_newsletter_issue(&invalid_body).await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "the api did not fail with '400 bad request' when the payload was {error_message}"
        )
    }
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act I - Login
    let login_body = serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password,
    });
    let response = app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/admin/dashboard");

    // Act II - send newsletter
    let newsletter_request_body = serde_json::json!({
        "title": "newsletter title",
        "content": "newsletter content"
    });
    let response = app.post_newsletter_issue(&newsletter_request_body).await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);
    // mock verifies that email was sent
}

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    // Arrange
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    // Act I - Login
    let login_body = serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password,
    });
    let response = app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/admin/dashboard");

    // Act II - send newsletter
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "content": "content"
    });
    let response = app.post_newsletter_issue(&newsletter_request_body).await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);
    // mock verifies that we have not received the newsletter email
}

async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscriptions(body)
        .await
        .error_for_status()
        .unwrap();

    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();
    app.get_confirmation_links(&email_request)
}

async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_link = create_unconfirmed_subscriber(app).await;
    reqwest::get(confirmation_link.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}
