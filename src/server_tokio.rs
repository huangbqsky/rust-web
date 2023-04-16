#![allow(dead_code, unused)]

use tokio::task::spawn;
use tokio::time::sleep;
use tokio::io::{Result, Error, ErrorKind, AsyncWriteExt, BufReader, AsyncBufReadExt, copy};
use std::time::Duration;
use tokio::sync::mpsc::unbounded_channel as channel;
use tokio::fs::File;
use tokio::net::{TcpListener, TcpStream};

/**
进化的 Http Server : 一 多线程 的程序改成异步程序：
转写后，功能完全相同，逻辑也相同，97行的程序只有25行不同。
1.替换所有 io 的 struct 和 function 为 tokio 提供的版本，基本只要改 use 就可以。
2.io 之外的阻塞操作比如 sleep 和 channel 也换成 tokio 的
3.替换后的函数调用，加后缀.await。更明确的说，语句值的类型是 Future 的，加后缀.await。
4.直接包括 .await 的函数定义，加前缀async
5.std::thread::spawn 改 tokio::task::spawn，参数的无参数闭包move || {} 改 async 块 async move {}
6.几处 API 修改。比如 tokio 的channel.recv() 返回Option而不是 Result 。tokio的BufReader 要 &mut stream 而不是 &stream 。write! 宏没有对应的异步实现，展开成 format! 宏和 write 函数调用。
7.main 函数已经被改成了 async ，再加上#[tokio::main]
 */
#[tokio::main]
async fn main() -> Result<()> {
    let (dispatch_sender, mut dispatch_receiver) = channel::<DispatchMessage>();

    let local_host = "127.0.0.1";
    let port = 20083;
    let listener = TcpListener::bind((local_host, port)).await?;
    let dispatch_sender1 = dispatch_sender.clone();

    let accept_loop = spawn(async move {
        while let Ok((stream, addr)) = listener.accept().await {
            dispatch_sender1.send(DispatchMessage::Connected(stream)).unwrap();
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

    //accept_loop.await?;
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

    let strsubs: Vec<_> = str.split(" ").collect();
    if strsubs.len() < 3 {
        return Err(Error::from(ErrorKind::InvalidInput));
    }
    let method = strsubs[0];
    let path = strsubs[1];

    let (path, query) = match path.find("?") {
        Some(pos) => (&path[..pos], &path[(pos+1)..]),
        None => (path, ""),
    };

    if query == "sleep" {
        sleep(Duration::new(4, 0)).await;
    }

    if path == "/" {
        stream.write("HTTP/1.1 200 OK\r\n\r\n<html><body>Welcome</body></html>".as_bytes()).await?;
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
