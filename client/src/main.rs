use crate::connection::Connection;
use bytes::BytesMut;
use message_core::message::{Packet, Request, Response};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use rsa::{
    Pkcs1v15Encrypt, RsaPublicKey,
    pkcs8::DecodePublicKey,
    rand_core::{OsRng, RngCore},
};
use std::{env::args, error::Error};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, stdin},
    net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
};

mod connection;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let connection = Connection::parse(args())?;

    let stream = TcpStream::connect(connection.get_host())
        .await
        .map_err(|e| {
            eprintln!("Could not connect to host {}: {}", connection.get_host(), e);
            e
        })?;

    println!("Successfully connected to: {}", stream.peer_addr()?);

    let (mut reader, mut writer) = stream.into_split();

    let public_key: RsaPublicKey;
    writer
        .write_all(&Request::Encryption.serialized().unwrap())
        .await
        .unwrap();

    let mut buf = BytesMut::with_capacity(1024);
    if let Ok(_) = reader.read_buf(&mut buf).await
        && let Response::Encryption(server_pub_key, token) = Response::deserialized(&buf).unwrap()
    {
        public_key = RsaPublicKey::from_public_key_der(&server_pub_key).unwrap();
        let mut secret = [0u8; 32];
        OsRng.fill_bytes(&mut secret);
        let enc_secret = public_key
            .encrypt(&mut OsRng, Pkcs1v15Encrypt, &secret[..])
            .expect("failed to encrypt");
        let enc_token = public_key
            .encrypt(&mut OsRng, Pkcs1v15Encrypt, &token[..])
            .expect("failed to encrypt");

        writer
            .write_all(
                &Request::EncryptionConfirm(enc_secret, enc_token)
                    .serialized()
                    .unwrap(),
            )
            .await
            .unwrap();

        let secret = Some(secret.to_vec());
        let mut seed = [0u8; 32];
        seed.copy_from_slice(&secret.as_ref().unwrap()[..]);
        let _nonce_generator_write = Some(ChaCha20Rng::from_seed(seed));
        let _nonce_generator_read = Some(ChaCha20Rng::from_seed(seed));
    };

    let reader_handle = tokio::spawn(async move { reading_loop(reader).await });
    let writing_handle = tokio::spawn(async move { writing_loop(writer, connection).await });

    reader_handle
        .await
        .map_err(|err| format!("Reader thread panicked: {:?}", err))?;
    writing_handle
        .await
        .map_err(|err| format!("Writer thread panicked: {:?}", err))?;

    Ok(())
}

async fn reading_loop(mut reader: OwnedReadHalf) {
    let mut buf = BytesMut::with_capacity(1024);

    loop {
        match reader.read_buf(&mut buf).await {
            Ok(0) => break,
            Ok(_) => {
                let response = Response::deserialized(&buf).unwrap();
                match response {
                    Response::Message(text) => {
                        println!("{text}");
                    }
                    Response::Join(_) => todo!(),
                    Response::Encryption(_, _) => todo!(),
                }
                buf.clear();
            }
            Err(e) => {
                eprintln!("Error reading from server: {}", e);
                break;
            }
        };
    }
}

async fn writing_loop(mut writer: OwnedWriteHalf, _connection: Connection) {
    let mut buf = BytesMut::with_capacity(1024);
    let mut stdin = stdin();
    loop {
        match stdin.read_buf(&mut buf).await {
            Ok(bytes) if bytes <= 2 => (),
            Ok(_) => {
                let string_msg = String::from_utf8_lossy(buf.trim_ascii_end()).to_string();
                buf.clear();

                let message = Request::Message(string_msg);
                if let Err(e) = writer.write_all(&message.serialized().unwrap()).await {
                    eprintln!("Could not write to server: {}", e);
                    break;
                }
            }
            Err(e) => eprintln!("Could not read buffer: {}", e),
        }
    }
}
