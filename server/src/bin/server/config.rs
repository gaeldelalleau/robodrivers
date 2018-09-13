extern crate serde_yaml;

use std::fs::{File, OpenOptions};
use std::io::Read;
use std::env;
use std::path::PathBuf;
use std::collections::HashMap;
use dirs;

use logging::LOGGER;


pub const CONFIG_FILENAME: &str = "config";
pub const SERIALIZED_FILES_EXTENSION: &str = "yaml";


#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Flag {
    pub score: u32,
    pub flag: String,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct TeamConfig {
    pub token: String,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub flags: Vec<Flag>,
    pub teams_config: HashMap<u32, TeamConfig>,
}

impl Config {
    fn get_config_dir(config_dir: Option<&str>) -> PathBuf {
        match config_dir {
            Some(d) => {
                let mut config_file = PathBuf::from(d);
                config_file.push(CONFIG_FILENAME);
                config_file.set_extension(SERIALIZED_FILES_EXTENSION);
                if config_file.exists() {
                    return PathBuf::from(d);
                } else {
                   warn!(logger!(), "Configuration file not found in specified directory");
                }
            },
            None => (),
        }
        match env::current_dir() {
            Ok(cur_dir) => {
                let mut config_file = cur_dir.clone();
                config_file.push(CONFIG_FILENAME);
                config_file.set_extension(SERIALIZED_FILES_EXTENSION);
                if config_file.exists() {
                    return cur_dir;
                }
            },
            Err(_) => (),
        }
        match env::current_exe() {
            Ok(exe_path) => exe_path.parent().expect("Unable to get parent directory of current executable path").to_path_buf(),
            Err(_) => match dirs::home_dir() {
                Some(home_dir) => home_dir,
                None => env::current_dir().expect("Unable to get home directory")
            },
        }
    }

    fn get_config_file(config_dir: &PathBuf) -> File {
        let mut config_file = config_dir.clone();
        config_file.push(CONFIG_FILENAME);
        config_file.set_extension(SERIALIZED_FILES_EXTENSION);

        debug!(logger!(), "Reading config file: {}", config_file.clone().display());
        OpenOptions::new().read(true).open(&config_file).expect("Unable to open config file")
    }

    pub fn new(config_dir: Option<&str>) -> (Config, PathBuf) {
        let mut serialized = String::new();
        let config_dir_path = Config::get_config_dir(config_dir);
        let mut config_file = Config::get_config_file(&config_dir_path);
        config_file.read_to_string(&mut serialized).expect("Unable to read config file");
        (Config::from_serialized(&serialized), config_dir_path)
    }

    fn from_serialized(serialized: &str) -> Config {
        serde_yaml::from_str::<Config>(serialized).expect("Unable to unserialize config file")
    }

    pub fn to_serialized(self: &Self) -> String {
        serde_yaml::to_string(self).expect("Unable to serialize config file")
    }
}

pub fn recreate_config() -> () {
    let mut config = Config::default();
    {
        let nb_flags = 3;
        let flags = &mut config.flags;
        for i in 0..nb_flags {
            flags.push(Flag { score: 1000*i, flag: format!("ICON{{XXXXXXXXXX{}}}", i).to_string() });
        }

        let nb_teams = 8;
        let teams_config = &mut config.teams_config;
        let team_tokens  = vec![ "XXX1", "XXX2", "XXX3", "XXX4", "XXX5", "XXX6", "XXX7", "XXX8" ];
        for i in 0..nb_teams {
            let team_id = i + 1;
            teams_config.insert(team_id, TeamConfig {
                token: team_tokens[i as usize].to_string(),
            });
        }
    }
    println!("{}", config.to_serialized());
}
