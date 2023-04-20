use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, HttpRequest};
use serde::Serialize;
 
// curl "127.0.0.1:8080"
#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
 
// curl "127.0.0.1:8080/echo" -H "Content-Type: application/json" -d "{"name":"dong"}"
#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}
 
// curl "127.0.0.1:8080/hey"
async fn hey() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

// curl "127.0.0.1:8080//name/{name}"
async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

#[derive(Serialize)]
struct Measurement {
    temperature: f32,
}

async fn current_temperature() -> impl Responder {
    web::Json(Measurement { temperature: 42.3 })
}
 
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello) // get service
            .service(echo) // post service
            .route("/hey", web::get().to(hey))
            .route("/name/{name}", web::get().to(greet))
            .route("/cur_temp", web::get().to(current_temperature))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}