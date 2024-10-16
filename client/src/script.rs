use std::error::Error;
use std::{fs, io};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};

pub struct ScriptsManager {
    pub scripts: Vec<Script>,
    path: String
}

impl<'a> ScriptsManager {
    pub fn load(path: String) -> Result<ScriptsManager, Box<dyn Error>> {

        let read = fs::read_dir(&path)?;

        let mut scripts = Vec::<Script>::new();
        for entry_result in read {
            let entry = entry_result?;
            scripts.push(Script::new(entry.path()));
        }

        Ok(ScriptsManager {
            scripts, path
        })
    }

    pub fn new_script(&self, name: String) -> Script {
        let path = Path::new(&self.path).join(name);
        let _ = File::create(&path);

        Script::new(path)
    }

    pub fn add_script(&mut self, script: Script) {
        self.scripts.push(script)
    }

    pub fn get_script(&self, name: String) -> Option<Script> {
        for script in &self.scripts {
            if let Some(script_name) = script.name() {
                if script_name == name {
                    return Some(script.clone())
                }
            }
        }

        None
    }
}


#[derive(Clone)]
pub struct Script {
    path: PathBuf
}

impl<'a> Script {

    fn new(path: PathBuf) -> Script {
        Script{
            path
        }
    }

    pub fn write(&self, data: String) -> io::Result<()> {
        File::open(&self.path)?.write_all(data.as_str().as_bytes())?;
        Ok(())
    }

    pub fn name(&self) -> Option<String> {
        let str = self.path.file_name()?.to_str()?;
        Some(String::from(str))
    }

    pub fn start(&self) -> Result<ScriptExecution, Box<dyn Error>> {
        let process = Command::new("cmd")
            .arg("/c")
            .arg(self.path.to_str().unwrap())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn().unwrap();


        Ok(ScriptExecution{
            process
        })
    }
}

pub struct ScriptExecution {
    process: Child
}

unsafe impl Send for ScriptExecution {}

impl ScriptExecution {
    pub fn is_alive(&mut self) -> bool {
        match self.process.try_wait() {
            Ok(Some(_)) => false,
            Ok(None) => true,
            Err(_) => false
        }
    }
}