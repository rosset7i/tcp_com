use crate::channel::{ChannelMessage, handle_channel};
use bytes::BytesMut;
use message_core::message::{Packet, Request};
use std::{error::Error, net::SocketAddr};
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, tcp::OwnedReadHalf},
    sync::mpsc::{Sender, channel},
};

mod channel;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    println!("Server listening on: {}", listener.local_addr()?);

    let (tx, rx) = channel::<ChannelMessage>(100);
    tokio::spawn(async move { handle_channel(rx).await });

    loop {
        let tx = tx.clone();
        let (stream, addr) = listener.accept().await?;
        println!("Accepted connection from: {addr}");

        let (reader, writer) = stream.into_split();
        if let Err(e) = tx.send(ChannelMessage::UserJoined((writer, addr))).await {
            eprintln!("Could not send message: {e}");
            continue;
        };

        tokio::spawn(async move { handle_connection(reader, tx, addr).await });
    }
}

async fn handle_connection(
    mut reader: OwnedReadHalf,
    tx: Sender<ChannelMessage>,
    current_client_addr: SocketAddr,
) {
    let mut buf = BytesMut::with_capacity(1024);

    loop {
        let bytes_read = match reader.read_buf(&mut buf).await {
            Ok(0) => break,
            Ok(val) => val,
            Err(e) => {
                eprintln!("Error while reading bytes: {e}");
                break;
            }
        };

        println!("Received {} bytes from {}", bytes_read, current_client_addr);

        let Ok(message) = Request::deserialized(&buf) else {
            eprintln!("Could not parse message, exiting...");
            break;
        };
        buf.clear();

        match message {
            Request::Message(text) => {
                if let Err(e) = tx
                    .send(ChannelMessage::Text((current_client_addr, text)))
                    .await
                {
                    eprintln!("Could not send message: {e}");
                    break;
                };
            }
            Request::Join(_) => {}
        }
    }

    println!("{} was disconnected!", current_client_addr);
}
