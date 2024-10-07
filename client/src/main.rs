use std::error::Error;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use log::{error, info, warn, LevelFilter};
use serde::Deserialize;
use crate::commands_executor::CommandsExecutor;
use crate::processes_watcher::{BlacklistFilter, ProcessesFilter, ProcessesWatcher};
use crate::server_api::ServerApi;

mod commands_executor;
mod server_api;
mod processes_watcher;

#[derive(Deserialize)]
struct FilterInfo {
    #[serde(rename="type")]
    filter_type: String,
    list: Vec<String>
}

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .init();

    let processes_watcher = Arc::new(Mutex::new(ProcessesWatcher::new()));

    thread::spawn({
        let processes_watcher_loop = processes_watcher.clone();
        move || {
            loop {
                sleep(Duration::from_micros(500));
                processes_watcher_loop.lock().unwrap().check();
            }
        }
    });

    let mut executor = CommandsExecutor::new();

    executor.add_handler_with_data(String::from("processes-watcher-set-filter"), |info: FilterInfo| {
        let mut watcher = processes_watcher.lock().unwrap();

        match info.filter_type.as_str() {
            "blacklist" => {
                let paths = info.list.iter().map(|x| Box::from(Path::new(x))).collect();
                watcher.set_filter(BlacklistFilter::new(paths))
            }
            t => warn!("Unknown filter type {}", t)
        }
    });

    loop {
        match ServerApi::new("ws://localhost:8000/client/socket") {
            Err(err) => error!("{}", err),
            Ok(mut server_api) => {
                info!("Client connected to server successfully");
                let result = process_commands(server_api, &executor);
                if let Err(err) = result {
                    error!("{}", err)
                }
            }
        }
    }
}

fn process_commands(mut server_api: ServerApi, executor: &CommandsExecutor) -> Result<(), Box<dyn Error>> {
    while server_api.alive {
        let mut command = server_api.next_command()?;
        if let Err(err) = executor.handle(&command.command, &mut command.data) {
            error!("{}", err)
        }
    }

    Ok(())
}