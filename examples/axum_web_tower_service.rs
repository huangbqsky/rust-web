use axum::{
    body::Body, http::Request, http::StatusCode, response::Response, routing::get_service, Router,
};
use futures::io;
use std::{convert::Infallible, net::SocketAddr};
use tower::service_fn;
use tower_http::services::ServeFile;

#[tokio::main]
async fn main() {
    let app = Router::new()
        // GET `/static/Cargo.toml` goes to a service from tower-http
        .route(
            "/static",
            get_service(ServeFile::new("Cargo.toml")).handle_error(|error: io::Error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {}", error),
                )
            }),
        )
        .route(
            // Any request to `/` goes to a some `Service`
            "/",
            get_service(service_fn(|_: Request<Body>| async {
                let res = Response::new(Body::from("Hi from `GET /`"));
                Ok::<_, Infallible>(res)
            })),
        );

    // run it with hyper on localhost:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
