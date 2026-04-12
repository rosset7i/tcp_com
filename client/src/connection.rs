use std::env::Args;

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: &str = "8080";

pub struct Connection {
    host: String,
    port: String,
    pub _username: String,
}

impl Connection {
    fn new(host: String, port: String, username: String) -> Self {
        Self {
            host,
            port,
            _username: username,
        }
    }

    pub fn parse(mut args: Args) -> Result<Self, String> {
        args.next(); // Consume first arg of executable path

        let username = args.next().ok_or("Username must be informed")?;
        let host = args.next().unwrap_or(DEFAULT_HOST.to_string());
        let port = args.next().unwrap_or(DEFAULT_PORT.to_string());

        Ok(Connection::new(host, port, username))
    }

    pub fn get_host(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
