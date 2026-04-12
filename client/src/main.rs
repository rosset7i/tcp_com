use crate::connection::Connection;
use std::{
    env::args,
    error::Error,
    io::{Read, Write, stdin},
    net::TcpStream,
    thread,
};

mod connection;

fn main() -> Result<(), Box<dyn Error>> {
    let connection = Connection::parse(args())?;

    let stream = TcpStream::connect(connection.get_host()).map_err(|e| {
        eprintln!("Could not connect to host {}: {}", connection.get_host(), e);
        e
    })?;

    println!("Successfully connected to: {}", stream.peer_addr()?);

    let (reader, writer) = (stream.try_clone()?, stream);

    let reader_handle = thread::spawn(move || reading_loop(reader));
    let writing_handle = thread::spawn(move || writing_loop(writer, connection));

    reader_handle
        .join()
        .map_err(|err| format!("Reader thread panicked: {:?}", err))?;
    writing_handle
        .join()
        .map_err(|err| format!("Writer thread panicked: {:?}", err))?;

    Ok(())
}

fn reading_loop(mut reader: TcpStream) {
    let mut buf = [0u8; 1024];

    loop {
        if let Ok(bytes) = reader.read(&mut buf) {
            let message = String::from_utf8_lossy(&buf[..bytes]);
            println!("[{}b]: {}", bytes, message);
        };
    }
}

fn writing_loop(mut writer: TcpStream, _connection: Connection) {
    let mut buf = [0u8; 1024];

    loop {
        match stdin().read(&mut buf) {
            Ok(bytes) if bytes <= 2 => (),
            Ok(bytes) => {
                if let Err(e) = writer.write_all(buf[..bytes].trim_ascii_end()) {
                    eprintln!("Could not write to server: {}", e);
                }
            }
            Err(e) => eprintln!("Could not read buffer: {}", e),
        }
    }
}
