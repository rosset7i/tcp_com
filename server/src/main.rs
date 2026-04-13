use std::{error::Error, net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        TcpListener,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::Mutex,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    println!("Server listening on: {}", listener.local_addr()?);

    let clients: Arc<Mutex<Vec<(OwnedWriteHalf, SocketAddr)>>> = Arc::new(Mutex::new(Vec::new()));

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("Accepted connection from: {}", addr);

        let (reader, writer) = stream.into_split();
        {
            let mut lock = clients.lock().await;
            lock.push((writer, addr));
        }

        let clients_for_thread = Arc::clone(&clients);

        tokio::spawn(async move { handle_connection(reader, clients_for_thread, addr).await });
    }
}

async fn handle_connection(
    mut reader: OwnedReadHalf,
    clients: Arc<Mutex<Vec<(OwnedWriteHalf, SocketAddr)>>>,
    current_client_addr: SocketAddr,
) {
    let mut buf = [0u8; 1024];

    loop {
        let bytes_read = match reader.read(&mut buf).await {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Error while reading bytes: {}", e);
                break;
            }
        };

        println!("Received {} bytes from {}", bytes_read, current_client_addr);

        let mut lock = clients.lock().await;
        let mut i = 0;
        while i < lock.len() {
            let (writer, addr) = &mut lock[i];

            if *addr != current_client_addr && writer.write_all(&buf[..bytes_read]).await.is_err() {
                eprintln!("Failed to write to {}", addr);
                lock.remove(i);
                continue;
            }

            i += 1;
        }
    }

    println!("{} was disconnected!", current_client_addr);
}
