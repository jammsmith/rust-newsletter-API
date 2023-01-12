use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("http://127.0.0.1:8000")?;

    newsletter_api::run(listener)?.await
}
