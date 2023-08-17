use std::net::TcpListener;

use secrecy::ExposeSecret;
use sqlx::PgPool;
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
    let connection = PgPool::connect(&configuration.database.connection_string().expose_secret())
        .await
        .expect("failed to connect to postgres");
    let address = format!("127.0.0.1:{port}", port = configuration.application_port);
    let listener = TcpListener::bind(address)?;
    info!(
        "listening on port {port}",
        port = configuration.application_port
    );
    run(listener, connection)?.await
}
