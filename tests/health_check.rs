use std::net::TcpListener;
use sqlx::{Connection, PgConnection};

use newsletter_api::startup::run;
use newsletter_api::configuration::get_configuration;

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(address + "/health_check")
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port.");

    let port = listener.local_addr().unwrap().port();

    let server = run(listener).expect("Failed to bind address");

    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn subscribe_returns_200_response_for_valid_data() {
    let app_address = spawn_app();

    let config = get_configuration().expect("Failed to read configuration.");

    let mut connection = PgConnection::connect(
        &config.database.connection_string()
    )
        .await
        .expect("Failed to connect to Postgres.");

    let client = reqwest::Client::new();

    let body = "name=john&20wayne&email=big_guns2&40gmail.com";

    let response = client
        .post(format!("{}/subscriptions", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());

    let subscribed = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(subscribed.email, "big_guns2@gmail.com");
    assert_eq!(subscribed.name, "john wayne");
}

#[tokio::test]
async fn subscribe_returns_400_response_when_data_is_missing() {
    let app_address = spawn_app();
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=john%20wayne", "email field missing"),
        ("email=big_guns&40gmail.com", "name field missing"),
        ("", "both name and email fields missing"),
    ];

    for (test_case, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", &app_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(test_case)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "API did not fail with 400 response code when payload had {}",
            error_message
        );
    }
}