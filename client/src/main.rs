use std::error::Error;
use std::sync::Arc;
use log::{error, info, LevelFilter};
use mac_address::get_mac_address;
use serde::Deserialize;
use system_shutdown::shutdown;
use crate::commands_executor::CommandsExecutor;
use crate::scripts::ScriptsManager;
use crate::server_api::ServerApi;

mod commands_executor;
mod server_api;
mod processes_watcher;
mod config;
mod scripts;
mod client;

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .init();

    let mut client = Arc::new(client::Client::new(String::from("./config.toml"), String::from("./scripts")).unwrap());

    let mac = get_mac_address().unwrap().unwrap();

    client.reg_handlers();
    client.spawn_process_watcher_loop();

    let socket_address = format!("ws://{}/client/socket", client.config.lock().unwrap().address);
    loop {
        match ServerApi::new(socket_address.as_str(), mac.to_string().as_str()) {
            Err(err) => error!("{}", err),
            Ok(server_api) => {
                info!("Client connected to server successfully");
                let result = process_commands(server_api, &client.executor.lock().unwrap());
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