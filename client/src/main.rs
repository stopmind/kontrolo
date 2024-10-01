use log::{error, info, warn, LevelFilter};
use crate::commands_executor::CommandsExecutor;
use crate::server_api::ServerApi;

mod commands_executor;
mod server_api;

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .init();

    let mut executor = CommandsExecutor::new();

    executor.add_handler_with_data(String::from("say-hi"), &|string: String| {
        println!("Hello, {}!", string);
    });

    executor.add_handler(String::from("connection-close"), &|| {
        warn!("Server connection closed.");
    });

    loop {
        let mut server_api = ServerApi::new("ws://localhost:8000/client/socket").unwrap();
        while server_api.alive {
            let mut command = server_api.next_command().unwrap();
            if let Err(err) = executor.handle(&command.command, &mut command.data) {
                error!("{}", err)
            }
        }
    }
}
