use sqlx::{Connection, PgConnection};
use std::net::TcpListener;

use newsletter_api::configuration::get_configuration;
use newsletter_api::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = get_configuration().expect("Failed to read configuration");

    let listener = TcpListener::bind(format!(
        "{}:{}",
        config.application.host, config.application.port
    ))?;

    let connection = PgConnection::connect(&config.database.connection_string())
        .await
        .expect("Failed to connect to Postgres DB");

    run(listener, connection)?.await
}
