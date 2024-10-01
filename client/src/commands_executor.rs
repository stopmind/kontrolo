use std::error;
use std::fmt::{Debug, Display, Formatter};
use dict::{Dict, DictIface};
use serde::de::DeserializeOwned;
use serde_json::Value;

#[derive(Debug)]
pub enum Error {
    NoHandler(String)
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoHandler(command) => write!(f, "No handler for command: {}", command)
        }
    }
}

impl error::Error for Error {}

pub struct CommandsExecutor<'a> {
    commands_handler: Dict<Box<dyn Fn(&mut Value) -> Result<(), serde_json::Error> + 'a>>
}

impl<'a> CommandsExecutor<'a> {
    pub fn new() -> Self {
        CommandsExecutor {
            commands_handler: Dict::new()
        }
    }

    pub fn handle(&self, command: &String, value: &mut Value) -> Result<(), Box<dyn error::Error>> {
        match self.commands_handler.get(command.as_str()) {
            None => Err(Box::new(Error::NoHandler(command.clone()))),
            Some(handler) => {
                if let Err(err) = handler(value) {
                    Err(err.into())
                } else {
                    Ok(())
                }
            }
        }
    }

    pub fn add_handler_with_data<T: DeserializeOwned>(&mut self, command: String, handler: &'a dyn Fn(T)) {
        self.commands_handler.add(command, Box::new(
            |value: &mut Value| {
                handler(serde_json::from_value(value.take())?);
                Ok(())
            }
        ));
    }

    pub fn add_handler(&mut self, command: String, handler: &'a dyn Fn()) {
        self.commands_handler.add(command, Box::new(
            |value: &mut Value| {
                handler();
                Ok(())
            }
        ));
    }
}