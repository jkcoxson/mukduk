// Jackson Coxson

const REVISION: &str = r#"A"#;
const DEFAULT: &str = r#"# Mukduk Config File
revision = "A"
bind = "127.0.0.1:8080"

[executable]
# The command to run
command = "server"
args = ["--port 8081"]
path = "."

[process]
# How long the process should run while inactive
inactivity = 600
# How long the user should wait before being redirected
load = 10
# Whether to pass through stdout, stdin and stderr
pipe = true
# The port the program listens to
port = 8081

"#;

use std::{fs::File, io::Write};

use log::{error, info};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub revision: String,
    pub bind: String,
    pub executable: Executable,
    pub process: Process,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Executable {
    pub command: String,
    pub args: Vec<String>,
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Process {
    pub inactivity: u16,
    pub load: u8,
    pub pipe: bool,
    pub port: u16,
}

impl Config {
    pub fn load(config_path: &str) -> std::io::Result<Config> {
        let mut config_path = config_path.to_string();
        if !config_path.ends_with(".toml") {
            config_path = format!("{}.toml", config_path);
        }
        match std::fs::read_to_string(&config_path) {
            Ok(contents) => match toml::from_str::<Config>(&contents) {
                Ok(c) => {
                    if c.revision != REVISION {
                        error!("Wrong revision of config file!");
                    }
                    Ok(c)
                }
                Err(e) => {
                    error!("Error parsing config: {}", e);
                    Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
                }
            },
            Err(e) => {
                error!("Config file {config_path} not found!");
                Err(e)
            }
        }
    }

    pub fn write() -> std::io::Result<()> {
        info!("Creating default config file");
        let default = DEFAULT.to_string();
        let mut file = File::create("default.toml")?;
        file.write_all(default.as_bytes())?;
        Ok(())
    }
}
