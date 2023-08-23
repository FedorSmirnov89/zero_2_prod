use std::{net::TcpListener, time::Duration};

use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use tracing::info;
use zero_2_prod::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("zero2prod", "info", std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("failed to read configuration");
    let connection = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .connect_lazy(&configuration.database.connection_string().expose_secret())
        .expect("failed to connect to postgres");
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
    run(listener, connection)?.await
}
