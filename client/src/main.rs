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

    reader_handle
        .join()
        .map_err(|err| format!("Reader thread panicked: {:?}", err))?;
    writing_handle
        .join()
        .map_err(|err| format!("Writer thread panicked: {:?}", err))?;

    Ok(())
}

fn reading_loop(mut reader: TcpStream) {
    let mut buf = [1u8; 1024];

    loop {
        if let Ok(bytes) = reader.read(&mut buf) {
            let message = String::from_utf8_lossy(&buf[..bytes]);
            println!("[{}b]: {}", bytes, message);
        };
    }
}

fn writing_loop(mut writer: TcpStream) {
    let mut buf = [1u8; 1024];

    loop {
        match stdin().read(&mut buf) {
            Ok(bytes) if bytes <= 2 => (),
            Ok(bytes) => {
                if let Err(e) = writer.write_all(sanitize_buffer(&buf[..bytes]).as_bytes()) {
                    eprintln!("Could not write to server: {}", e);
                }
            }
            Err(e) => eprintln!("Could not read buffer: {}", e),
        }
    }
}

fn sanitize_buffer(buf: &[u8]) -> String {
    let message = String::from_utf8_lossy(buf);

    message
        .strip_suffix("\r\n")
        .or(message.strip_suffix("\n"))
        .unwrap_or(&message)
        .to_string()
}
