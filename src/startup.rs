use std::{net::TcpListener, time::Duration};

use actix_web::{
    dev::Server,
    web::{self, Data},
    App, HttpServer,
};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::info;
use tracing_actix_web::TracingLogger;

use crate::{
    configuration::{DataBaseSettings, Settings},
    email_client::EmailClient,
    routes::{
        health_check::health_check, subscriptions::subscribe, subscriptions_confirm::confirm,
    },
};

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let connection = get_connection_pool(&configuration.database).await;

        let sender_email = configuration
            .email_client
            .sender()
            .expect("invalid sender email address");
        let timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            &configuration.email_client.base_url,
            sender_email,
            configuration.email_client.authorization_token,
            timeout,
        );

        let address = format!(
            "{host}:{port}",
            port = configuration.application.port,
            host = configuration.application.host
        );
        let listener = TcpListener::bind(address)?;
        info!(
            "listening on port {port}",
            port = configuration.application.port
        );
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            connection,
            email_client,
            configuration.application.base_url,
        )?;
        let application = Self { server, port };
        Ok(application)
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub async fn get_connection_pool(configuration: &DataBaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}

// We need this one to provide the url as app context
pub struct ApplicationBaseUrl(pub String);

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);
    let base_url = Data::new(ApplicationBaseUrl(base_url));
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
