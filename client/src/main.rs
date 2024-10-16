use std::error::Error;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use log::{error, info, warn, LevelFilter};
use mac_address::{get_mac_address, MacAddressIterator};
use serde::__private::de::Content;
use serde::Deserialize;
use system_shutdown::shutdown;
use crate::commands_executor::CommandsExecutor;
use crate::config::Config;
use crate::processes_watcher::{BlacklistFilter, ProcessesFilter, ProcessesWatcher};
use crate::script::ScriptsManager;
use crate::server_api::ServerApi;

mod commands_executor;
mod server_api;
mod processes_watcher;
mod config;
mod script;

#[derive(Deserialize)]
struct FilterInfo {
    #[serde(rename="type")]
    filter_type: String,
    list: Vec<String>
}

#[derive(Deserialize)]
struct ScriptSet {
    name: String,
    content: String
}

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .init();

    let config = Config::read("./config.toml").unwrap();

    let mac = get_mac_address().unwrap().unwrap();

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

    let mut scripts_manager = Mutex::new(ScriptsManager::load(String::from("scripts")).unwrap());

    let mut executor = CommandsExecutor::new();

    executor.add_handler(String::from("shutdown"), || {
        if let Err(err) = shutdown() {
            error!("{}", err);
        }
    });

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

    executor.add_handler_with_data(String::from("scripts-new"), |name: String| {
        info!("Added new script {}", &name);
        scripts_manager.lock().unwrap().add_script(scripts_manager.lock().unwrap().new_script(name));
    });

    executor.add_handler_with_data(String::from("scripts-set"), |set_info: ScriptSet| {
        info!("Script updated {}", &set_info.name);
        scripts_manager.lock().unwrap().get_script(set_info.name).unwrap()
            .write(set_info.content).unwrap();
    });

    executor.add_handler_with_data(String::from("scripts-run"), |name: String| {
        info!("Script started {}", &name);
        if let Some(script) = scripts_manager.lock().unwrap().get_script(name) {
            if let Ok(mut execution) = script.start() {
                thread::spawn(move || {
                    while execution.is_alive() {
                        sleep(Duration::from_micros(500));
                    }


                    info!("Script finished.")
                });
            }
        }
    });

    executor.add_handler_with_data(String::from("scripts-remove"), |name: String| {
        info!("Script removed {}", name);
    });

    let socket_address = format!("ws://{}/client/socket", config.address);
    loop {
        match ServerApi::new(socket_address.as_str(), mac.to_string().as_str()) {
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