use bytes::BytesMut;
use message_core::message::{Packet, Request, Response};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use rsa::{
    Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey,
    pkcs8::EncodePublicKey,
    rand_core::{OsRng, RngCore},
};
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

type ConnectedUser = (Sender<Response>, SocketAddr);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let private_key = RsaPrivateKey::new(&mut OsRng, 1024)?;
    let public_key = Arc::new(RsaPublicKey::from(&private_key));
    let arc_for_private = Arc::new(private_key);

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    println!("Server listening on: {}", listener.local_addr()?);

    let clients: Arc<Mutex<Vec<ConnectedUser>>> = Arc::new(Mutex::new(Vec::new()));

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("Accepted connection from: {}", addr);

        let (sender, receiver) = channel::<Response>(100);
        let (reader, writer) = stream.into_split();
        {
            let mut lock = clients.lock().await;
            lock.push((sender, addr));
        }

        let clients_for_thread = Arc::clone(&clients);

        let key = Arc::clone(&public_key);
        let p_key = Arc::clone(&arc_for_private);

        tokio::spawn(async move {
            handle_connection(key, p_key, reader, clients_for_thread, addr).await
        });
        tokio::spawn(async move { handle_broadcast(receiver, writer).await });
    }
}

async fn handle_connection(
    public_key: Arc<RsaPublicKey>,
    private_key: Arc<RsaPrivateKey>,
    mut reader: OwnedReadHalf,
    clients: Arc<Mutex<Vec<ConnectedUser>>>,
    current_client_addr: SocketAddr,
) {
    let mut buf = BytesMut::with_capacity(1024);

    let mut secret;
    let mut nonce_generator;
    let mut token = [0u8; 32];
    OsRng.fill_bytes(&mut token);

    loop {
        let bytes_read = match reader.read_buf(&mut buf).await {
            Ok(0) => break,
            Ok(val) => val,
            Err(e) => {
                eprintln!("Error while reading bytes: {}", e);
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
                let message = format!("{}: {}", current_client_addr, text);
                let teste = Response::Message(message);
                let lock = clients.lock().await;
                for (sender, addr) in lock.iter() {
                    if *addr != current_client_addr {
                        let _ = sender.send(teste.clone()).await;
                    };
                }
            }
            Request::Encryption => {
                let lock = clients.lock().await;
                let (sender, _) = lock
                    .iter()
                    .find(|(_, addr)| *addr == current_client_addr)
                    .unwrap();

                let _ = sender
                    .send(Response::Encryption(
                        public_key.to_public_key_der().unwrap().as_ref().to_vec(),
                        token.to_vec(),
                    ))
                    .await;
            }
            Request::Join(_) => todo!(),
            Request::EncryptionConfirm(client_public_key, encript_token) => {
                let client_token = private_key
                    .decrypt(Pkcs1v15Encrypt, &encript_token)
                    .unwrap();
                if client_token != token {
                    panic!("change this later");
                }

                secret = private_key
                    .decrypt(Pkcs1v15Encrypt, &client_public_key)
                    .unwrap();
                let mut seed = [0u8; 32];
                seed.copy_from_slice(&secret);
                nonce_generator = ChaCha20Rng::from_seed(seed);
                println!("Encryption successful");
            }
        }
    }

    println!("{} was disconnected!", current_client_addr);
}

async fn handle_broadcast(mut receiver: Receiver<Response>, mut writer: OwnedWriteHalf) {
    while let Some(response) = receiver.recv().await {
        let bytes = response.serialized().unwrap();
        if writer.write_all(&bytes).await.is_err() {
            let _ = writer.shutdown().await;
            receiver.close();
        }
    }
}
