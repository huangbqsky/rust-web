use axum::{
    extract::{ContentLengthLimit, Multipart, Path},
    http::header::{HeaderMap, HeaderName, HeaderValue},
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Router,
};

use rand::prelude::random;
use std::fs::read;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;

const SAVE_FILE_BASE_PATH: &str = "/Users/huangbq/Downloads/upload";
/**
 * 
将演示axum如何实现图片上传（注：其它类型的文件原理相同），一般来说要考虑以下几个因素：
1. 文件上传的大小限制
2. 文件上传的类型限制（仅限指定类型：比如图片）
3. 防止伪装mimetype进行攻击（比如：把.js文件改后缀变成.jpg伪装图片上传，早期有很多这类攻击)
另外，上传图片后，还可以让浏览器重定向到上传后的图片（当然，仅仅只是演示技术实现，实际应用中并非一定要这样）
 */
// 上传表单
async fn show_upload() -> Html<&'static str> {
    Html(
        r#"
        <!doctype html>
        <html>
            <head>
            <meta charset="utf-8">
                <title>上传文件(仅支持图片上传)</title>
            </head>
            <body>
                <form action="/save_image" method="post" enctype="multipart/form-data">
                    <label>
                    上传文件(仅支持图片上传)：
                        <input type="file" name="file">
                    </label>
                    <button type="submit">上传文件</button>
                </form>
            </body>
        </html>
        "#,
    )
}

// 上传图片 ，20M限制
async fn save_image(
    ContentLengthLimit(mut multipart): ContentLengthLimit<Multipart, { /* 20M 限制*/ 1024 * 1024 * 20 }>,
) -> Result<(StatusCode, HeaderMap), String> {
    if let Some(file) = multipart.next_field().await.unwrap() {
        //文件类型
        let content_type = file.content_type().unwrap().to_string();

        //校验是否为图片(出于安全考虑)
        if content_type.starts_with("image/") {
            //根据文件类型生成随机文件名(出于安全考虑)
            let rnd = (random::<f32>() * 1000000000 as f32) as i32;
            //提取"/"的index位置
            let index = content_type
                .find("/")
                .map(|i| i)
                .unwrap_or(usize::max_value());
            //文件扩展名
            let mut ext_name = "xxx";
            if index != usize::max_value() {
                ext_name = &content_type[index + 1..];
            }
            //最终保存在服务器上的文件名
            let save_filename = format!("{}/{}.{}", SAVE_FILE_BASE_PATH, rnd, ext_name);

            //文件内容
            let data = file.bytes().await.unwrap();

            //辅助日志
            println!("filename:{},content_type:{}", save_filename, content_type);

            //保存上传的文件
            tokio::fs::write(&save_filename, &data)
                .await
                .map_err(|err| err.to_string())?;

            //上传成功后，显示上传后的图片
            return redirect(format!("/show_image/{}.{}", rnd, ext_name)).await;
        }
    }

    //正常情况，走不到这里来
    println!("{}", "没有上传文件或文件格式不对");

    //当上传的文件类型不对时，下面的重定向有时候会失败(感觉是axum的bug)
    return redirect(format!("/upload")).await;
}

/**
 * 显示图片
 */
async fn show_image(Path(id): Path<String>) -> (HeaderMap, Vec<u8>) {
    let index = id.find(".").map(|i| i).unwrap_or(usize::max_value());
    //文件扩展名
    let mut ext_name = "xxx";
    if index != usize::max_value() {
        ext_name = &id[index + 1..];
    }
    let content_type = format!("image/{}", ext_name);
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_str(&content_type).unwrap(),
    );
    let file_name = format!("{}/{}", SAVE_FILE_BASE_PATH, id);
    (headers, read(&file_name).unwrap())
}

/**
 * 重定向
 */
async fn redirect(path: String) -> Result<(StatusCode, HeaderMap), String> {
    let mut headers = HeaderMap::new();
    //重设LOCATION，跳到新页面
    headers.insert(
        axum::http::header::LOCATION,
        HeaderValue::from_str(&path).unwrap(),
    );
    //302重定向
    Ok((StatusCode::FOUND, headers))
}

#[tokio::main]
async fn main() {
    // Set the RUST_LOG, if it hasn't been explicitly defined
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "example_sse=debug,tower_http=debug")
    }
    tracing_subscriber::fmt::init();

    // our router
    let app = Router::new()
        .route("/upload", get(show_upload))
        .route("/save_image", post(save_image))
        .route("/show_image/:id", get(show_image))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("listening on {}", addr);
    tracing::debug!("listening on {}", addr);
    // run it with hyper on localhost:3000
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
