use std::{
    error::Error,
    io::Read,
    net::{SocketAddr, TcpListener, TcpStream},
    thread,
};

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Server listening on: {}", listener.local_addr()?);

    for stream in listener.incoming() {
        let stream = stream?;
        let peer = stream.peer_addr()?;
        println!("Accepted connection from: {}", peer);

        let reader = stream;

        thread::spawn(move || handle_connection(reader, peer));
    }
    Ok(())
}

fn handle_connection(mut reader: TcpStream, peer: SocketAddr) {
    let mut buf = [0u8; 1024];

    loop {
        let bytes_read = match reader.read(&mut buf) {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Error while reading bytes: {}", e);
                break;
            }
        };

        let message = String::from_utf8_lossy(&buf[..bytes_read]);
        println!(
            "Received {} bytes from {}: {}",
            bytes_read,
            reader.peer_addr().unwrap(),
            message
        );
    }

    println!("{} was disconnected!", peer);
}
