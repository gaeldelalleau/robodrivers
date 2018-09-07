extern crate serde_yaml;

use std::fs::{File, OpenOptions};
use std::io::Read;
use std::env;
use std::path::PathBuf;
use dirs;

use logging::LOGGER;


pub const CONFIG_FILENAME: &str = "config";
pub const SERIALIZED_FILES_EXTENSION: &str = "yaml";

lazy_static! {
    pub static ref WORKING_DIRECTORY: PathBuf = {
        match env::current_exe() {
            Ok(exe_path) => exe_path.parent().expect("Unable to get parent directory of current executable path").to_path_buf(),
            Err(_) => match dirs::home_dir() {
                Some(home_dir) => home_dir,
                None => env::current_dir().expect("Unable to get current directory")
            },
        }
    };
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Flag {
    pub score: i32,
    pub flag: String,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Config {
    pub flags: Vec<Flag>,
}

impl Config {
    fn get_config_file() -> File {
        let mut config_file = WORKING_DIRECTORY.clone();
        config_file.push(CONFIG_FILENAME);
        config_file.set_extension(SERIALIZED_FILES_EXTENSION);

        debug!(logger!(), "Reading config file: {}", config_file.clone().display());
        OpenOptions::new().read(true).open(&config_file).expect("Unable to open config file")
    }

    pub fn new() -> Config {
        let mut serialized = String::new();
        Config::get_config_file().read_to_string(&mut serialized).expect("Unable to read config file");
        Config::from_serialized(&serialized)
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
        let flags = &mut config.flags;
        for _ in 0..3 {
            flags.push(Flag { score: 1000, flag: "ICON{XXXXXXXXXX}".to_string() });
        }
    }
    println!("{}", config.to_serialized());
}


