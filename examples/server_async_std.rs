// #![allow(dead_code, unused)]
/**
 * 这里可以 diff 比一下 async-std 和 tokio 的不同，你会发现除了改下 use ，调整几处 API，几乎没改什么。
 * 占了一半代码量的 handle_connection 函数，两边甚至完全一样。
 */
use async_std::task::spawn;
use async_std::task::sleep;
use async_std::io::{Result, Error, ErrorKind, BufReader, copy, prelude::*};
use std::time::Duration;
use async_std::channel::unbounded as channel;
use async_std::fs::File;
use async_std::net::{TcpListener, TcpStream};

#[async_std::main]
async fn main() -> Result<()> {
    let (dispatch_sender, dispatch_receiver) = channel::<DispatchMessage>();

    let local_host = "127.0.0.1";
    let port = 20083;
    let listener = TcpListener::bind((local_host, port)).await?;
    let dispatch_sender1 = dispatch_sender.clone();
    let _accept_loop = spawn(async move {
        while let Ok((stream, addr)) = listener.accept().await {
            println!("TcpListener accept: {} ", addr);
            dispatch_sender1.send(DispatchMessage::Connected(stream)).await.unwrap();
        }
    });
    println!("server started at http://{}:{}/ serving files in {:?}", local_host, port, std::env::current_dir().unwrap_or_default());

    while let Ok(dispatch_message) = dispatch_receiver.recv().await {
        match dispatch_message {
            DispatchMessage::Connected(stream) => {
                let dispatch_sender = dispatch_sender.clone();
                spawn(async move {
                    if let Ok(RequestResult::Quit) = handle_connection(stream).await {
                        dispatch_sender.send(DispatchMessage::Quit).await.unwrap();
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
        stream.write("HTTP/1.1 200 OK\r\n\r\n<html><body>Welcome async_std Server</body></html>".as_bytes()).await?;
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
                eprintln!("{}", err);
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
