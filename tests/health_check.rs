use std::net::TcpListener;
use zero2prod::startup;
use sqlx::PgPool;
use zero2prod::configuration::get_configuration;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool
}


async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");

    let port = listener.local_addr().unwrap().port();

    let configuration = get_configuration().expect("Failed to get configuration");

    let connection_string = configuration.database.connection_string();

    let connection_pool = PgPool::connect(&connection_string).await.expect("Failed to connect to postgres");

    let server = startup::run(listener, connection_pool.clone()).expect("Failed to bind address");

    let address = format!("http://127.0.0.1:{}", port);

    let _ = tokio::spawn(server);

    
     TestApp {
        address,
        db_pool: connection_pool,
     }
}

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    //arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    //act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");
    //assert

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("Select email, name FROM subscriptions").fetch_one(&app.db_pool).await.expect("failed to fetch saved subscription");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");

}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    //arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    //act

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}",
            error_message
        );
    }
}
