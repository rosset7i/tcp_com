use std::{
    error::Error,
    io::{Read, Write, stdin},
    net::TcpStream,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:8080")?;
    let mut buf = [1u8; 1024];

    loop {
        let bytes_read = stdin().read(&mut buf)?;
        let bytes_written = stream.write(&buf[..bytes_read])?;
        println!("Written {} bytes to server", bytes_written);
    }
}
