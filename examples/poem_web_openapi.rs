use poem::{listener::TcpListener, Route};
use poem_openapi::{param::Query, payload::PlainText, OpenApi, OpenApiService};

struct Api;

/// 定义了一个路径为/hello的API，它接受一个名为name的URL参数，并且返回一个字符串作为响应内容。
/// name参数的类型是Option<String>，意味着这是一个可选参数。
#[OpenApi]
impl Api {
    #[oai(path = "/hello", method = "get")]
    async fn index(&self, name: Query<Option<String>>) -> PlainText<String> {
        match name.0 {
            Some(name) => PlainText(format!("hello, {}!", name)),
            None => PlainText("hello!".to_string()),
        }
    }
}

// 运行以下代码后，用浏览器打开http://localhost:3000就能看到Swagger UI，你可以用它来浏览API的定义并且测试它们。
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    
    // 创建API服务
    let api_service =
        OpenApiService::new(Api, "Hello World", "1.0").server("http://localhost:3000/api");

    // 开启Swagger UI
    let ui = api_service.swagger_ui();

    // 创建 Router, 并指定api的根路径为 /api, Swagger UI的路径为 /
    let app = Route::new().nest("/api", api_service).nest("/", ui);
     // 创建一个TCP监听器
    let listener = TcpListener::bind("127.0.0.1:3000");
    // 启动服务器，并指定api的根路径为 /api，Swagger UI的路径为 /
    poem::Server::new(listener)
        .run(app)
        .await
}
