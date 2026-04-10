use std::{
    error::Error,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Server listening on: {}", listener.local_addr()?);

    let clients: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));

    for stream in listener.incoming() {
        let stream = stream?;
        let peer = stream.peer_addr()?;
        println!("Accepted connection from: {}", peer);

        {
            let mut lock = clients.lock().unwrap();
            lock.push(stream.try_clone()?);
        }

        let reader = stream;

        let clients_for_thread = Arc::clone(&clients);
        thread::spawn(move || handle_connection(reader, clients_for_thread, peer));
    }
    Ok(())
}

fn handle_connection(mut reader: TcpStream, clients: Arc<Mutex<Vec<TcpStream>>>, peer: SocketAddr) {
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
        println!("[{}b] {}: {}", bytes_read, peer, message);

        let mut lock = clients.lock().unwrap();

        lock.retain(|mut stream| {
            if stream.peer_addr().unwrap() == peer {
                return true;
            }

            if let Err(e) = stream.write_all(message.as_bytes()) {
                eprintln!("Failed to write to {}: {}", stream.peer_addr().unwrap(), e);
                return false;
            }
            true
        });
    }

    println!("{} was disconnected!", peer);
}
