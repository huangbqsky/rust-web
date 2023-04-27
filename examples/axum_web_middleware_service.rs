use axum::{
    http::Request,
    middleware::Next,
    response::Response,
    Router, routing::get,
    ServiceExt, // for `into_make_service`
};
use tower::Layer;

async fn rewrite_request_uri<B>(req: Request<B>, next: Next<B>) -> Response {
    // ...
    next.run(req).await
}

async fn handler() { /* ... */ }

#[tokio::main]
async fn main() {
    // this can be any `tower::Layer`
    let middleware = axum::middleware::from_fn(rewrite_request_uri);

    let app = Router::new().route("/", get(handler));

    // apply the layer around the whole `Router`
    // this way the middleware will run before `Router` receives the request
    let app_with_middleware = middleware.layer(app);

    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app_with_middleware.into_make_service()) // 全局起作用
        .await
        .unwrap();
}
