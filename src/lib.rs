use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer};
use std::net::TcpListener;

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[derive(serde::Deserialize)]
struct FormData {
    email: String,
    name: String,
}

async fn subscribe(_form: web::Form<FormData>) -> HttpResponse {
    println!("Name: {} Email: {}", _form.name, _form.email);
    HttpResponse::Ok().finish()
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let address = listener.local_addr().unwrap();

    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();

    println!("Listening on {}", address);
    Ok(server)
}
