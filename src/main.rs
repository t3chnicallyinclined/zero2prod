use std::net::TcpListener;
use zero2prod::startup::run;
use zero2prod::configuration::get_configuration;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let config = get_configuration().expect("Failed to get configuration");


    let connection_pool = PgPool::connect(&config.database.connection_string()).await.expect("Failed to connect to postgres");

    let address = format!("127.0.0.1:{}", config.application_port);

    let listener = TcpListener::bind(address).expect("Failed to bind port");

    run(listener, connection_pool)?.await
}
