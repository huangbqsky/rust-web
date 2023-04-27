use axum::{
    Router,
    routing::get,
    extract::State,
    http::Request, response::IntoResponse,
};
use tower::{Layer, Service};
use std::task::{Context, Poll};

#[derive(Clone, Debug)]
struct AppState {
    state: i32,
}

#[derive(Clone, Debug)]
struct MyLayer {
    state: AppState,
}

impl<S> Layer<S> for MyLayer {
    type Service = MyService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MyService {
            inner,
            state: self.state.clone(),
        }
    }
}

#[derive(Clone, Debug)]
struct MyService<S> {
    inner: S,
    state: AppState,
}

impl<S, B> Service<Request<B>> for MyService<S>
where
    S: Service<Request<B>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        // Do something with `self.state`.
        self.state.state = self.state.state + 1;
        // See `axum::RequestExt` for how to run extractors directly from
        // a `Request`.
        self.inner.call(req)
    }
}

async fn handler(state: State<AppState>) -> impl IntoResponse {
    println!("{:?}", state);
    format!("{:?}", state)
}

#[tokio::main]
async fn main() {
    let state = AppState {state: 0};

    let app = Router::new()
        .route("/", get(handler))
        .layer(MyLayer { state: state.clone() })
        .with_state(state);

    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

