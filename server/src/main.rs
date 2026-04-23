use message_core::message::Message;
use std::{error::Error, net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        TcpListener,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::{
        Mutex,
        mpsc::{Receiver, Sender, channel},
    },
};

type ConnectedUser = (Sender<Vec<u8>>, SocketAddr);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    println!("Server listening on: {}", listener.local_addr()?);

    let clients: Arc<Mutex<Vec<ConnectedUser>>> = Arc::new(Mutex::new(Vec::new()));

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("Accepted connection from: {}", addr);

        let (sender, receiver) = channel::<Vec<u8>>(100);
        let (reader, writer) = stream.into_split();
        {
            let mut lock = clients.lock().await;
            lock.push((sender, addr));
        }

        let clients_for_thread = Arc::clone(&clients);

        tokio::spawn(async move { handle_connection(reader, clients_for_thread, addr).await });
        tokio::spawn(async move { handle_broadcast(receiver, writer).await });
    }
}

async fn handle_connection(
    mut reader: OwnedReadHalf,
    clients: Arc<Mutex<Vec<ConnectedUser>>>,
    current_client_addr: SocketAddr,
) {
    let mut buf = [0u8; 1024];

    loop {
        let bytes_read = match reader.read(&mut buf).await {
            Ok(0) => break,
            Ok(val) => val,
            Err(e) => {
                eprintln!("Error while reading bytes: {}", e);
                break;
            }
        };

        println!("Received {} bytes from {}", bytes_read, current_client_addr);

        match Message::deserialized(&buf[..bytes_read]) {
            Message::Text(text) => {
                let message = format!("{}: {}", current_client_addr, text).into_bytes();

                let lock = clients.lock().await;
                for (sender, addr) in lock.iter() {
                    if *addr != current_client_addr {
                        let _ = sender.send(message.clone()).await;
                    };
                }
            }
            Message::JoinRequest(_) => {}
        }
    }

    println!("{} was disconnected!", current_client_addr);
}

async fn handle_broadcast(mut receiver: Receiver<Vec<u8>>, mut writer: OwnedWriteHalf) {
    while let Some(bytes) = receiver.recv().await {
        if writer.write_all(&bytes).await.is_err() {
            let _ = writer.shutdown().await;
            receiver.close();
        }
    }
}
