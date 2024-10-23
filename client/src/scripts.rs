use std::fmt::{Debug, Display, Formatter};
use std::{fs, io};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use log::info;
use mlua::Lua;
use mlua::prelude::LuaResult;
use crate::client::Client;
use crate::scripts::Error::NotExist;

#[derive(Debug)]
pub enum Error {
    NotExist(String),
    AlreadyExist(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NotExist(id) => write!(f, "Script with id {} not exists", id),
            Error::AlreadyExist(id) => write!(f, "Script with id {} already exists", id)
        }
    }
}

impl std::error::Error for Error {}

type Result<T> = std::result::Result<T, Error>;

pub struct ScriptsManager {
    path: String
}

impl ScriptsManager {
    pub fn new(path: String) -> Self {
        ScriptsManager {
            path
        }
    }

    pub fn get_script(&self, id: &String) -> Result<Script> {

        let path = Path::new(&self.path).join(id);

        if !path.exists() {
            return Err(NotExist(id.clone()))
        }
        
        Ok(Script{path})
    }

    pub fn new_script(&self, id: &String) -> Result<Script> {
        let path = Path::new(&self.path).join(id);

        if path.exists() {
            return Err(Error::AlreadyExist(id.clone()))
        }

        File::create(&path).unwrap();

        Ok(Script{path})
    }
}

pub struct Script {
    path: PathBuf
}

impl Script {
    pub fn remove(&self) -> io::Result<()> {
        fs::remove_file(&self.path)
    }

    pub fn update(&self, content: &String) -> io::Result<()> {
        OpenOptions::new()
            .write(true)
            .open(&self.path)?
            .write_all(content.as_bytes())
    }

    pub fn exec(&self, client: &Client) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let lua = Lua::new();

        lua.globals().set("log", lua.create_function(log)?)?;

        let content = fs::read_to_string(&self.path)?;

        lua.load(content).exec()?;

        Ok(())
    }
}

fn log(_: &Lua, string: String) -> LuaResult<()> {
    info!("{}", string);
    Ok(())
}