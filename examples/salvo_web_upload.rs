use std::fs::create_dir_all;
use std::path::Path;

use salvo::prelude::*;
use salvo::size_limiter::max_size;

#[handler]
async fn index(res: &mut Response) {
    res.render(Text::Html(INDEX_HTML));
}

// 文件上传
#[handler]
async fn upload(req: &mut Request, res: &mut Response) {
    let file = req.file("file").await;
    if let Some(file) = file {
        let dest = format!("temp/{}", file.name().unwrap_or_else(|| "file".into()));
        tracing::debug!(dest = %dest, "upload file");
        if let Err(e) = std::fs::copy(&file.path(), Path::new(&dest)) {
            res.set_status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(format!("file not found in request: {}", e.to_string()));
        } else {
            res.render(format!("File uploaded to {}", dest));
        }
    } else {
        res.set_status_code(StatusCode::BAD_REQUEST);
        res.render("file not found in request");
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    tracing::info!("Listening on http://127.0.0.1:7878");
    create_dir_all("temp").unwrap();
    tracing::info!("create temp directory");
    let router = Router::new()
        .get(index)  // http://127.0.0.1:7878
        .push(Router::new()
                .hoop(max_size(1024 * 1024 * 10)) // 提供对请求上传文件大小限制的中间件.
                .path("limited")
                .post(upload),
        )
        .push(Router::new().path("unlimit").post(upload));
    Server::new(TcpListener::bind("127.0.0.1:7878"))
        .serve(router)
        .await;
}

static INDEX_HTML: &str = r#"<!DOCTYPE html>
<html>
    <head>
        <title>Upload file</title>
    </head>
    <body>
        <h1>Upload file</h1>
        <form action="/unlimit" method="post" enctype="multipart/form-data">
            <h3>Unlimit</h3>
            <input type="file" name="file" />
            <input type="submit" value="upload" />
        </form>
        <form action="/limited" method="post" enctype="multipart/form-data">
            <h3>Limited 10MiB</h3>
            <input type="file" name="file" />
            <input type="submit" value="upload" />
        </form>
    </body>
</html>
"#;
