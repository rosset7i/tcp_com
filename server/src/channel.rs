use std::net::SocketAddr;
use tokio::{io::AsyncWriteExt, net::tcp::OwnedWriteHalf, sync::mpsc::Receiver};

type ConnectedUser = (OwnedWriteHalf, SocketAddr);
type TextMessage = (SocketAddr, String);

#[derive(Debug)]
pub enum ChannelMessage {
    Text(TextMessage),
    UserJoined(ConnectedUser),
}

pub async fn handle_channel(mut rx: Receiver<ChannelMessage>) {
    let mut clients: Vec<ConnectedUser> = Vec::new();

    loop {
        if let Some(channel_message) = rx.recv().await {
            match channel_message {
                ChannelMessage::Text((sender_addr, text)) => {
                    let message = format!("{}: {}", sender_addr, text);
                    for (writer, _) in clients.iter_mut().filter(|(_, addr)| *addr != sender_addr) {
                        writer.write_all(message.as_bytes()).await.unwrap();
                    }
                }
                ChannelMessage::UserJoined(client) => clients.push(client),
            }
        }
    }
}
