#![allow(dead_code, unused)]

use std::time::Duration;

use tokio::task::spawn;
use tokio::time::sleep;
use tokio::io::{Result, Error, ErrorKind, AsyncWriteExt, BufReader, AsyncBufReadExt, copy};
use tokio::sync::mpsc::unbounded_channel as channel;
use tokio::fs::File;
use tokio::net::{TcpListener, TcpStream};
use tokio::select;

/**
异步实现真正的优雅停机

技术细节：
1.加一个 channel kill_switch 对这种只发一次的，tokio 的 oneshot 语义更清晰，当然 tokio 的 bounded_channel unbounded_channel 也可以用
2.accept_loop 内用 select! 处理多个异步事件
3.主线程结束前 用 kill_switch 发消息给 accept_loop 让其停止， accept_loop.await 类似于线程的 join 等待异步任务退出。
 */

#[tokio::main]
async fn main() -> Result<()> {
    let (dispatch_sender, mut dispatch_receiver) = channel::<DispatchMessage>();
    let (kill_switch, kill_switch_receiver) = tokio::sync::oneshot::channel::<()>();

    let local_host = "127.0.0.1";
    let port = 20083;
    let listener = TcpListener::bind((local_host, port)).await?;
    let dispatch_sender1 = dispatch_sender.clone();
    let accept_loop = spawn(async move {
        select! {
            _ = async {
                while let Ok((stream, addr)) = listener.accept().await {
                    println!("TcpListener accept: {} ", addr);
                    dispatch_sender1.send(DispatchMessage::Connected(stream)).unwrap();
                }
            } => {}
            _ = kill_switch_receiver => {}
        }
    });
    println!("server started at http://{}:{}/ serving files in {:?}", local_host, port, std::env::current_dir().unwrap_or_default());

    while let Some(dispatch_message) = dispatch_receiver.recv().await {
        match dispatch_message {
            DispatchMessage::Connected(stream) => {
                let dispatch_sender = dispatch_sender.clone();
                spawn(async move {
                    if let Ok(RequestResult::Quit) = handle_connection(stream).await {
                        dispatch_sender.send(DispatchMessage::Quit).unwrap();
                    }
                });
            }
            DispatchMessage::Quit => { break; }
        }
    }

    kill_switch.send(()).unwrap();
    accept_loop.await?;
    Ok(())
}

#[derive(Debug)]
enum DispatchMessage {
    Connected(TcpStream),
    Quit,
}

enum RequestResult {
    Ok,
    Quit,
}

async fn handle_connection(mut stream: TcpStream) -> Result<RequestResult> {
    let mut str = String::new();
    BufReader::new(&mut stream).read_line(&mut str).await?;
    println!("request: {}", str.trim());
    let strsubs: Vec<_> = str.split(" ").collect();
    if strsubs.len() < 3 {
        return Err(Error::from(ErrorKind::InvalidInput));
    }
    let method = strsubs[0];
    let path = strsubs[1];
    println!("method: {method} , path: {path}");

    let (path, query) = match path.find("?") {
        Some(pos) => (&path[..pos], &path[(pos+1)..]),
        None => (path, ""),
    };

    if query == "sleep" {
        sleep(Duration::new(4, 0)).await;
    }

    if path == "/" {
        stream.write("HTTP/1.1 200 OK\r\n\r\n<html><body>Welcome Tokio Server</body></html>".as_bytes()).await?;
    } else {
        let relative_path = match path.strip_prefix("/") {
            Some(p) => p,
            None => path,
        };
        match File::open(relative_path).await {
            Ok(mut f) => {
                stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).await?;
                copy(&mut f, &mut stream).await?;
            }
            Err(err) => {
                stream.write(format!("HTTP/1.1 404 NOT FOUND\r\n\r\n<html><body>Not Found {}</body></html>", path).as_bytes()).await?;
            }
        }
    }
    stream.flush().await?;

    if query == "quit" {
        return Ok(RequestResult::Quit);
    }
    return Ok(RequestResult::Ok);
}
