use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
 
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
async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}
 
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}