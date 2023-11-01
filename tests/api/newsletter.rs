use std::time::Duration;

use wiremock::{
    matchers::{any, method, path},
    Mock, ResponseTemplate,
};

use fake::faker::internet::en::SafeEmail;
use fake::faker::name::en::Name;
use fake::Fake;

use crate::helpers::{assert_is_redirect_to, spawn_app, ConfirmationLinks, TestApp};

#[tokio::test]
async fn concurrent_form_submission_is_handled_gracefully() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(2)))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act - Submit two newsletter forms concurrently
    let newsletter_request_body = serde_json::json!({
        "title": "newsletter title",
        "content": "newsletter content",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });
    let response1 = app.post_newsletter_issue(&newsletter_request_body);
    let response2 = app.post_newsletter_issue(&newsletter_request_body);
    let (response1, response2) = tokio::join!(response1, response2);

    // Assert
    assert_eq!(response1.status(), response2.status());
    assert_eq!(
        response1.text().await.unwrap(),
        response2.text().await.unwrap()
    );
    app.dispatch_all_pending_emails().await;
    // Mock verifies that the newletter was sent only once
}

#[tokio::test]
async fn newsletter_creation_is_idempotent() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act I
    let newsletter_request_body = serde_json::json!({
        "title": "newsletter title",
        "content": "newsletter content",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });
    let response = app.post_newsletter_issue(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    // Act II - Follow the redirect
    let html_page = app.get_publish_newsletter_html().await;
    assert!(html_page.contains("The newsletter issue has been published."));

    // Act III - submit the newsletter again
    let response = app.post_newsletter_issue(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    // Act IV - Follow the redirect (again)
    let html_page = app.get_publish_newsletter_html().await;
    dbg!(&html_page);
    assert!(html_page.contains("The newsletter issue has been published."));
    app.dispatch_all_pending_emails().await;

    // Mock verifies that we've sent only one email when it is dropped
}

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
        "content": "newsletter content",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });
    let response = app.post_newsletter_issue(&newsletter_request_body).await;

    // Assert
    assert_is_redirect_to(&response, "/admin/newsletters");
    let html_page = app.get_publish_newsletter_html().await;
    assert!(html_page.contains("The newsletter issue has been published."));
    app.dispatch_all_pending_emails().await;
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
        "content": "content",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });
    let response = app.post_newsletter_issue(&newsletter_request_body).await;

    // Assert
    assert_is_redirect_to(&response, "/admin/newsletters");
    let html_page = app.get_publish_newsletter_html().await;
    assert!(html_page.contains("The newsletter issue has been published."));
    // mock verifies that we have not received the newsletter email
}

async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let name: String = Name().fake();
    let email: String = SafeEmail().fake();
    let body = serde_urlencoded::to_string(&serde_json::json!({
        "name": name,
        "email": email
    }))
    .unwrap();

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
