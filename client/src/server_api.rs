use std::error::Error;
use std::net::TcpStream;
use std::str::FromStr;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tungstenite::{connect, Message, WebSocket};
use tungstenite::stream::MaybeTlsStream;
use url::Url;

#[derive(Deserialize, Serialize)]
pub struct Command {
    pub command: String,
    pub data: Value
}

#[derive(Deserialize, Serialize)]
struct HelloMessage {
    mac: String
}

impl HelloMessage {
    fn new() -> HelloMessage {
        HelloMessage {
            mac: String::from("test mac")
        }
    }
}

pub struct ServerApi {
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
    pub alive: bool
}

impl ServerApi {
    pub fn new(url: &str) -> Result<ServerApi, Box<dyn Error>> {
        let url = Url::from_str(url)?;
        let (mut socket, _) = connect(&url)?;

        let hello_message = serde_json::to_string(&HelloMessage::new())?;
        if let Err(err) = socket.send(Message::Text(hello_message)) {
            let _ = socket.close(None);
            return Err(err.into())
        }

        Ok(ServerApi{
            socket,
            alive: true
        })
    }

    pub fn next_command(&mut self) -> Result<Command, Box<dyn Error>> {
        loop {
            let message = match self.socket.read() {
                Ok(msg) => msg,
                Err(err) => return Err(err.into())
            };

            let string: String = match message {
                Message::Text(str) => str,
                Message::Close(..) => {
                    self.alive = false;
                    return Ok(Command{
                        command: String::from("connection-close"),
                        data: Value::Null,
                    })
                },
                _ => continue
            };

            return Ok(serde_json::from_str(string.as_str())?);
        }
    }
}