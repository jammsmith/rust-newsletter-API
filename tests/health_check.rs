use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;

use newsletter_api::configuration::{get_configuration, DatabaseSettings};
use newsletter_api::startup::run;

struct TestApp {
    pub app_address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port.");

    let mut config = get_configuration().expect("Failed to read configuration.");
    config.database.database_name = Uuid::new_v4().to_string();

    let db_pool = configure_database(&config.database).await;

    let port = listener.local_addr().unwrap().port();

    let server = run(listener, db_pool.clone()).expect("Failed to bind address");

    let _ = tokio::spawn(server);

    TestApp {
        app_address: format!("http://127.0.0.1:{}", port),
        db_pool,
    }
}

// Spin up a new database with random name for tests
async fn configure_database(db_config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&db_config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_config.database_name).as_str())
        .await
        .expect("Failed to create database");

    let connection_pool = PgPool::connect(&db_config.connection_string())
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

#[tokio::test]
async fn health_check_works() {
    let TestApp { app_address, .. } = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .get(app_address + "/health_check")
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_response_for_valid_data() {
    let TestApp {
        app_address,
        db_pool,
    } = spawn_app().await;

    let client = reqwest::Client::new();

    let body = "name=john%20wayne&email=big_guns%40gmail.com";

    let response = client
        .post(format!("{}/subscriptions", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());

    let subscribed = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(subscribed.email, "big_guns@gmail.com");
    assert_eq!(subscribed.name, "john wayne");
}

#[tokio::test]
async fn subscribe_returns_400_response_when_data_is_missing() {
    let TestApp { app_address, .. } = spawn_app().await;

    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=john%20wayne", "email field missing"),
        ("email=big_guns%40gmail.com", "name field missing"),
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
