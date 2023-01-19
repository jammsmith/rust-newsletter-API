use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::PgConnection;
use std::net::TcpListener;

use crate::routes::{health_check, subscriptions};

pub fn run(listener: TcpListener, db_connection: PgConnection) -> Result<Server, std::io::Error> {
    let db_connection_pointer = web::Data::new(db_connection);

    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check::health_check))
            .route("/subscriptions", web::post().to(subscriptions::subscribe))
            .app_data(db_connection_pointer.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
