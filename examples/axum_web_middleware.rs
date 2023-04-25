use std::net::SocketAddr;
use axum::{routing::get, Router};
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

#[tokio::main]
async fn main() {
    let middleware_stack = ServiceBuilder::new()
        // add high level tracing of requests and responses
        .layer(TraceLayer::new_for_http())
        // compression responses
        .layer(CompressionLayer::new())
        // convert the `ServiceBuilder` into a `tower::Layer`;
        .into_inner();

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .layer(middleware_stack);

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
