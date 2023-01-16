use std::net::TcpListener;

use newsletter_api::startup::run;
use newsletter_api::configuration::get_configuration;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = get_configuration().expect("Failed to read configuration");

    run(
        TcpListener::bind(
            format!("{}:{}", config.application.host, config.application.port)
        )?
    )?
    .await
}
