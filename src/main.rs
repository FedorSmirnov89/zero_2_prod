use std::fmt::{Debug, Display};

use tokio::task::JoinError;
use zero_2_prod::{
    configuration::get_configuration,
    issue_delivery_worker::run_worker_until_stopped,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber("zero2prod", "info", std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("failed to read configuration");
    let application = Application::build(configuration.clone()).await?;
    let application_task = tokio::task::spawn(application.run_until_stopped());
    let worker_task = tokio::task::spawn(run_worker_until_stopped(configuration));

    tokio::select! {
        o = application_task => report_exit("API", o),
        o = worker_task => report_exit("Background Worker", o)
    }

    Ok(())
}

fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            tracing::info!("{task_name} has exited")
        }
        Ok(Err(e)) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{task_name} failed",
            )
        }
        Err(e) => {
            tracing::error!(error.cause_chain = ?e, error.message = %e, "{task_name} task failed to complete")
        }
    }
}
