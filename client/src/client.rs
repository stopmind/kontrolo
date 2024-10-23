use std::error::Error;
use std::path::Path;
use std::sync::{Arc, LockResult, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use log::{error, warn};
use serde::Deserialize;
use system_shutdown::shutdown;
use crate::commands_executor::CommandsExecutor;
use crate::config::Config;
use crate::processes_watcher::{BlacklistFilter, ProcessesWatcher};
use crate::scripts::ScriptsManager;

pub struct Client<'a> {
    pub processes_watcher: Mutex<ProcessesWatcher<'a>>,
    pub executor: Mutex<CommandsExecutor<'a>>,
    pub config: Mutex<Config>,
    pub scripts_manager: Mutex<ScriptsManager>
}

unsafe impl Send for Client<'_> {}
unsafe impl Sync for Client<'_> {}

#[derive(Deserialize)]
struct FilterInfo {
    #[serde(rename="type")]
    filter_type: String,
    list: Vec<String>
}

#[derive(Deserialize)]
struct ScriptUpdate {
    id: String,
    content: String
}

impl Client<'_> {
    pub fn new(config_path: String, scripts_path: String) -> Result<Self, Box<dyn Error>> {
        let client = Client {
            config: Mutex::new(Config::read(config_path)?),
            processes_watcher: Mutex::new(ProcessesWatcher::new()),
            executor: Mutex::new(CommandsExecutor::new()),
            scripts_manager: Mutex::new(ScriptsManager::new(scripts_path))
        };

        Ok(client)
    }
}
impl Client<'static> {
    pub fn spawn_process_watcher_loop(self: &Arc<Self>) {
        let sub_arc = Arc::clone(self);

        thread::spawn(move || {
            loop {
                sleep(Duration::from_micros(500));
                sub_arc.processes_watcher.lock().unwrap().check();
            }
        });
    }

    pub fn reg_handlers(self: &Arc<Self>) {
        let mut executor = self.executor.lock().unwrap();

        executor.add_handler(String::from("shutdown"), || {
            if let Err(err) = shutdown() {
                error!("{}", err);
            }
        });

        {
            let sub_arc = Arc::clone(self);
            executor.add_handler_with_data(String::from("processes-watcher-set-filter"), move |info: FilterInfo| {
                let mut watcher = sub_arc.processes_watcher.lock().unwrap();

                match info.filter_type.as_str() {
                    "blacklist" => {
                        let paths = info.list.iter().map(|x| Box::from(Path::new(x))).collect();
                        watcher.set_filter(BlacklistFilter::new(paths))
                    }
                    t => warn!("Unknown filter type {}", t)
                }
            });
        }

        {
            let sub_arc = Arc::clone(self);
            executor.add_handler_with_data(String::from("scripts-update"), move |update: ScriptUpdate| {
                let mut scripts = sub_arc.scripts_manager.lock().unwrap();

                let script = match scripts.get_script(&update.id) {
                    Ok(script) => script,
                    Err(_) => scripts.new_script(&update.id).unwrap()
                };

                if let Err(err) = script.update(&update.content) {
                    error!("{}", err)
                }
            });
        }

        {
            let sub_arc = Arc::clone(self);
            executor.add_handler_with_data(String::from("scripts-remove"), move |id: String| {
                let mut scripts = sub_arc.scripts_manager.lock().unwrap();

                match scripts.get_script(&id) {
                    Err(err) => error!("{}", err),
                    Ok(script) => {
                        if let Err(err) = script.remove() {
                            error!("{}", err);
                        }
                    }
                }
            });
        }

        {
            let sub_arc = Arc::clone(self);
            executor.add_handler_with_data(String::from("scripts-exec"), move |id: String| {
                let sub_sub_arc = Arc::clone(&sub_arc);

                thread::spawn(move || {
                    let mut scripts = sub_sub_arc.scripts_manager.lock().unwrap();

                    match scripts.get_script(&id) {
                        Err(err) => error!("{}", err),
                        Ok(script) => {
                            if let Err(err) = script.exec(sub_sub_arc.as_ref()) {
                                error!("{}", err);
                            }
                        }
                    }
                });
            });
        }
    }
}