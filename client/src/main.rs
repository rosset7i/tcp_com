use crate::connection::Connection;
use bytes::BytesMut;
use message_core::message::{Packet, Request};
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

    let (reader, writer) = stream.into_split();

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
                println!("{}", String::from_utf8_lossy(&buf));
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

    loop {
        match stdin().read_buf(&mut buf).await {
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
