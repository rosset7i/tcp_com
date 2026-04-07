use std::{
    error::Error,
    io::{Read, Write, stdin},
    net::TcpStream,
    thread,
};

fn main() -> Result<(), Box<dyn Error>> {
    let stream = TcpStream::connect("127.0.0.1:8080")?;
    println!("Successfully connected to: {}", stream.peer_addr()?);

    let (reader, writer) = (stream.try_clone()?, stream);

    let reader_handle = thread::spawn(|| reading_loop(reader));
    let writing_handle = thread::spawn(|| writing_loop(writer));

    reader_handle.join().unwrap();
    writing_handle.join().unwrap();

    Ok(())
}

fn reading_loop(mut reader: TcpStream) {
    let mut buf = [1u8; 1024];

    loop {
        let bytes_read = reader.read(&mut buf).unwrap(); // TODO: Remove this unwrap()
        let message = String::from_utf8_lossy(&buf[..bytes_read]);
        println!("Read {} bytes from server: {}", bytes_read, message);
    }
}

fn writing_loop(mut writer: TcpStream) {
    let mut buf = [1u8; 1024];

    loop {
        let bytes_read = stdin().read(&mut buf).unwrap();
        let _bytes_written = writer.write(&buf[..bytes_read]).unwrap();
    }
}
