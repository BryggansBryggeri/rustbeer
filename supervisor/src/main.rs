#![forbid(unsafe_code)]
use bryggio_lib::config;
use bryggio_lib::pub_sub::{nats_client::run_nats_server, PubSubClient, PubSubError};
use bryggio_lib::supervisor::{Supervisor, SupervisorError};
use std::env;
use std::path::{Path, PathBuf};

fn config_file_from_args() -> Result<PathBuf, SupervisorError> {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        2 => Ok(Path::new(&args[1]).to_path_buf()),
        _ => Err(SupervisorError::Cli(
            "Usage: bryggio-supervisor <path_to_config_file>".into(),
        )),
    }
}

fn main() -> Result<(), SupervisorError> {
    let config_file = config_file_from_args()?;
    let config = match config::Config::try_new(&config_file) {
        Ok(config) => config,
        Err(err) => {
            return Err(PubSubError::Configuration(format!(
                "Invalid config file '{}'. Error: {}.",
                config_file.to_string_lossy(),
                err.to_string()
            ))
            .into());
        }
    };
    let mut nats_server_child = run_nats_server(&config.nats)?;
    let supervisor = Supervisor::init_from_config(config)?;
    supervisor.client_loop()?;
    nats_server_child
        .kill()
        .map_err(|err| PubSubError::Server(err.to_string()).into())
}
