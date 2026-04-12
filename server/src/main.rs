use std::{
    error::Error,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:8080")?;
    println!("Server listening on: {}", listener.local_addr()?);

    let clients: Arc<Mutex<Vec<(TcpStream, SocketAddr)>>> = Arc::new(Mutex::new(Vec::new()));

    for stream in listener.incoming() {
        let stream = stream?;
        let peer = stream.peer_addr()?;
        println!("Accepted connection from: {}", peer);

        {
            let mut lock = clients.lock().unwrap_or_else(|e| e.into_inner());
            lock.push((stream.try_clone()?, stream.peer_addr()?));
        }

        let reader = stream;

        let clients_for_thread = Arc::clone(&clients);
        thread::spawn(move || handle_connection(reader, clients_for_thread, peer));
    }
    Ok(())
}

fn handle_connection(
    mut reader: TcpStream,
    clients: Arc<Mutex<Vec<(TcpStream, SocketAddr)>>>,
    peer: SocketAddr,
) {
    let mut buf = [0u8; 1024];

    loop {
        let bytes_read = match reader.read(&mut buf) {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Error while reading bytes: {}", e);
                break;
            }
        };

        println!("Received {} bytes from {}", bytes_read, peer);

        let mut lock = clients.lock().unwrap_or_else(|e| e.into_inner());

        lock.retain_mut(|(stream, addr)| {
            if *addr == peer {
                return true;
            }

            if let Err(e) = stream.write_all(&buf[..bytes_read]) {
                eprintln!("Failed to write to {}: {}", addr, e);
                return false;
            }
            true
        });
    }

    println!("{} was disconnected!", peer);
}
