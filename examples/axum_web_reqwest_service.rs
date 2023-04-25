use axum::{
    error_handling::HandleError,
    http::{Response, StatusCode},
    Router,
};

#[tokio::main]
async fn main() {
    // this service might fail with `reqwest::Error`
    let fallible_service = tower::service_fn(|_req| async {
        let body = can_fail().await?;
        Ok::<_, reqwest::Error>(Response::new(body))
    });

    // Since fallible_service can fail with 'reqwest::Error',
    // you can't directly route it to "/".
    // Use route_service to convert any errors
    // encountered into a response
    let app = Router::new().route_service("/", HandleError::new(fallible_service, handle_error));

    let addr = "127.0.0.1:3000";
    println!("listening on {}", addr);
    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn can_fail() -> Result<String, reqwest::Error> {
    // send a request to a site that doesn't exist
    // so we can see the handler fail
    let body = reqwest::get("https://www.abcdth.org").await?.text().await?;
    Ok(body)
}

async fn handle_error(err: reqwest::Error) -> (StatusCode, String) {
    return (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Something went wrong: {}", err),
    );
}
