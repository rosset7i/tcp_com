use std::{error::Error, net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, tcp::OwnedReadHalf},
    sync::{
        Mutex,
        mpsc::{self, UnboundedSender},
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    println!("Server listening on: {}", listener.local_addr()?);

    let clients: Arc<Mutex<Vec<(UnboundedSender<Vec<u8>>, SocketAddr)>>> =
        Arc::new(Mutex::new(Vec::new()));

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("Accepted connection from: {}", addr);

        let (sender, mut receiver) = mpsc::unbounded_channel::<Vec<u8>>();
        let (reader, mut writer) = stream.into_split();
        {
            let mut lock = clients.lock().await;
            lock.push((sender, addr));
        }

        let clients_for_thread = Arc::clone(&clients);

        tokio::spawn(async move { handle_connection(reader, clients_for_thread, addr).await });
        tokio::spawn(async move {
            loop {
                let teste = receiver.recv().await.unwrap();
                writer.write_all(&teste).await.unwrap();
            }
        });
    }
}

async fn handle_connection(
    mut reader: OwnedReadHalf,
    clients: Arc<Mutex<Vec<(UnboundedSender<Vec<u8>>, SocketAddr)>>>,
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

        lock.retain(|(sender, addr)| {
            if *addr != current_client_addr {
                return sender.send(buf[..bytes_read].to_vec()).is_ok();
            };

            true
        })
    }

    println!("{} was disconnected!", current_client_addr);
}
