use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
mod class_loader;
mod java_class;
mod attributes;
mod utils;
mod errors;
use class_loader::*;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;
        println!("Connection accpeted ");

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = match socket.read(&mut buf).await {
                    // socket closed
                    Ok(0) => {
                        println!("Socket closed");
                         return
                    },
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                // Write the data back
                if let Ok(s) = String::from_utf8(buf.to_vec()) {
                    println!("{s}");
                }
                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
}
