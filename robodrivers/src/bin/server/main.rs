#![feature(plugin)]
#![plugin(tarpc_plugins)]

extern crate robodrivers;

#[macro_use]
extern crate lazy_static;
extern crate dirs;
extern crate clap;
extern crate ws;

#[macro_use]
extern crate slog;
extern crate slog_term;
extern crate slog_async;

extern crate futures;
#[macro_use]
extern crate tarpc;
extern crate tokio_core;

#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

use std::process;
use std::env;
use std::path::{Path, PathBuf};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use std::thread;
use std::time;
use std::sync::Mutex;
use clap::{Arg, ArgMatches, App};
use slog::Drain;
use ws::{listen, Handler, Sender, Message, CloseCode};

use tarpc::future::server;
use tarpc::util::{FirstSocketAddr, Never};
use tokio_core::reactor;

use commands::FutureServiceExt as CommandsExt;


const CONFIG_FILENAME: &str = "config";
const BASE_GAME_STATE_FILENAME: &str = "game_state_base";
const CURRENT_GAME_STATE_FILENAME: &str = "game_state_{}";
const SERIALIZED_FILES_EXTENSION: &str = "yaml";

const MAX_RPC_REQUEST_SIZE: u64 = 8192; // in bytes


lazy_static! {
    static ref LOGGER: Mutex<Vec<slog::Logger>> = Mutex::new(vec![]);
}

lazy_static! {
    static ref WORKING_DIRECTORY: PathBuf = {
        match env::current_exe() {
            Ok(exe_path) => exe_path.parent().expect("Unable to get parent directory of current executable path").to_path_buf(),
            Err(_) => match dirs::home_dir() {
                Some(home_dir) => home_dir,
                None => env::current_dir().expect("Unable to get current directory")
            },
        }
    };
}

macro_rules! logger {
    () => ( LOGGER.lock().expect("Error while trying to acquire lock for LOGGER")[0] )
}

pub mod commands {
    service! {
        rpc action(name: String) -> String;
        rpc flags() -> String;
    }
}

#[derive(Clone)]
struct CommandsServer;

impl commands::FutureService for CommandsServer {
    type ActionFut = Result<String, Never>;

    fn action(&self, name: String) -> Self::ActionFut {
        Ok(format!("Hello, {}!", name))
    }

    type FlagsFut = Result<String, Never>;

    fn flags(&self) -> Self::FlagsFut {
        Ok(format!("Hello!"))
    }
}

fn start_rpc_server(_config: &Config) -> () {
    trace!(logger!(), "Starting RPC server");
    let reactor = reactor::Core::new().expect("Unable to create a new Reactor");
    let mut options = server::Options::default();
    options = options.max_payload_size(MAX_RPC_REQUEST_SIZE);
    let (_handle, server) = CommandsServer.listen("0:3011".first_socket_addr(),
                                  &reactor.handle(),
                                  options)
                          .expect("Unable to listen on socket for RPC server");
    reactor.handle().spawn(server);
    trace!(logger!(), "RPC server started");

    /*
    use futures::Future;
    use tarpc::future::client;
    use tarpc::future::client::ClientExt;

    let options = client::Options::default().handle(reactor.handle());
    reactor.run(commands::FutureClient::connect(handle.addr(), options)
            .map_err(tarpc::Error::from)
            .and_then(|client| client.action("MOVE NORTH".to_string()))
            .map(|resp| debug!(logger, "Got response {}", resp)))
            .expect("Error while doing RPC communication");
    */
}

struct Server {
    out: Sender,
}

impl Handler for Server {

    fn on_message(&mut self, _msg: Message) -> ws::Result<()> {
        self.out.send("wut?")
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        match code {
            CloseCode::Normal => debug!(logger!(), "The client is done with the connection."),
            CloseCode::Away   => debug!(logger!(), "The client is leaving the site."),
            _ => info!(logger!(), "The client encountered an error: {}", reason),
        }
    }
}

fn start_ws_server() -> () {
    trace!(logger!(), "Starting WebSocket server");
    let _ws_server = thread::Builder::new().name("ws_server".to_owned()).spawn(move || {
        listen("0:3012", |out| Server { out: out } ).expect("Unable to listen on socket for Websocket server");
    }).expect("Unable to spawn new thread for Websocket server");
    trace!(logger!(), "WebSocket server started");
}

struct GameEngine {
    game_state : GameState,
    current_game_state_file: File,
}

impl GameEngine {

    fn get_game_state_file(game_id: u32) -> File {
        let mut base_game_state_file = WORKING_DIRECTORY.clone();
        base_game_state_file.push(BASE_GAME_STATE_FILENAME);
        base_game_state_file.set_extension(SERIALIZED_FILES_EXTENSION);

        let mut current_game_state_file = WORKING_DIRECTORY.clone();
        current_game_state_file.push(CURRENT_GAME_STATE_FILENAME.replace("{}", &game_id.to_string()));
        current_game_state_file.set_extension(SERIALIZED_FILES_EXTENSION);

        debug!(logger!(), "Reading config files from {}", WORKING_DIRECTORY.clone().display());
        debug!(logger!(), "Current game state file is: {}", current_game_state_file.clone().display());

        match OpenOptions::new().read(true).write(true).open(&current_game_state_file) {
            Ok(file) => file,
            Err(_) => {
                info!(logger!(), "No game state file found for game {}, will create a fresh one", game_id);
                fs::copy(&base_game_state_file, &current_game_state_file).expect("Unable to copy base game state file to current game state file");
                match OpenOptions::new().read(true).write(true).open(&current_game_state_file) {
                    Ok(file) => file,
                    Err(_) => panic!("Couldn't copy base game state file {} to file {}", base_game_state_file.display(), current_game_state_file.display()),
                }
            }
        }
    }

    fn new() -> GameEngine {
        let game_id = 0;
        let mut file = GameEngine::get_game_state_file(game_id);

        let mut serialized = String::new();
        file.read_to_string(&mut serialized).expect("Unable to read game state file");
        let mut game_state = GameState::from_serialized(&serialized);
        game_state.id = game_id;

        GameEngine { game_state: game_state, current_game_state_file: file }
    }

    fn save_game_state(self: &mut Self) -> () {
        // TODO: lock gamestate mutex for the duration of this function
        trace!(logger!(), "Saving game state to file");
        let serialized = self.game_state.to_serialized();
        self.current_game_state_file.seek(SeekFrom::Start(0)).expect("Unable to seek at start of game state file");
        self.current_game_state_file.write_all(serialized.as_bytes()).expect("Unable to write to game state file");
        let cursor_pos = self.current_game_state_file.seek(SeekFrom::Current(0)).expect("Unable to get cursor position of game state file");
        self.current_game_state_file.set_len(cursor_pos).expect("Unable to truncate game state file at cursor position");
        self.current_game_state_file.flush().expect("Unable to flush game state file");
        trace!(logger!(), "Game state saved to file");
    }

    fn step(self: &mut Self) -> () {
        // TODO: lock gamestate mutex for the duration of this function
        // TODO: simulate game: update game state from team actions, and delete all
        //       team actions
        self.game_state.tick += 1;
    }

    fn start(self: &mut Self) -> () {
        trace!(logger!(), "Starting game loop");
        loop {
            thread::sleep(time::Duration::from_millis(200));
            self.step();
            if self.game_state.tick % 10 == 0 {
                self.save_game_state();
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum Resource {
    GAS(u32)
}

#[derive(Serialize, Deserialize, Debug)]
struct Producer {
    resource: Resource,
    cooldown: u32,
}

#[derive(Serialize, Deserialize, Debug)]
enum Item {
    BASE,
    RESOURCE(Resource),
    PRODUCER(Producer),
}

#[derive(Serialize, Deserialize, Debug)]
enum Block {
    WALL,
    OPEN,
}

#[derive(Serialize, Deserialize, Debug)]
struct Cell {
    block: Block,
    items: Vec<Item>,
}

#[derive(Default, Serialize, Deserialize, Debug)]
struct Map {
    cells: Vec<Vec<Cell>>,
}

#[derive(Serialize, Deserialize, Debug)]
enum Direction {
    NORTH,
    SOUTH,
    EAST,
    WEST,
}

#[derive(Serialize, Deserialize, Debug)]
enum State {
    MOVING(Direction),
    STOPPED
}

impl Default for State {
    fn default() -> State { State::STOPPED }
}


#[derive(Default, Serialize, Deserialize, Debug)]
struct Car {
    x: i32,
    y: i32,
    health: i32,
    resources: i32,
    state: State,
}

#[derive(Default, Serialize, Deserialize, Debug)]
struct Team {
    id: u32,
    name: String,
    color: String,
    score: i32,
    token: String,
    car: Car,
}

#[derive(Default, Serialize, Deserialize, Debug)]
struct GameState {
    id: u32,
    map: Map,
    teams: Vec<Team>,
    tick: u32,
}

impl GameState {
    fn new() -> GameState {
        GameState::default()
    }

    fn from_serialized(serialized: &str) -> GameState {
        serde_yaml::from_str::<GameState>(serialized).expect("Unable to deserialize game state")
    }

    fn to_serialized(self: &Self) -> String {
        serde_yaml::to_string(self).expect("Unable to serialize game state")
    }
}


#[derive(Default, Serialize, Deserialize, Debug)]
struct Flag {
    score: i32,
    flag: String,
}

#[derive(Default, Serialize, Deserialize, Debug)]
struct Config {
    flags: Vec<Flag>,
}

impl Config {
    fn get_config_file() -> File {
        let mut config_file = WORKING_DIRECTORY.clone();
        config_file.push(CONFIG_FILENAME);
        config_file.set_extension(SERIALIZED_FILES_EXTENSION);

        debug!(logger!(), "Reading config file: {}", config_file.clone().display());
        OpenOptions::new().read(true).open(&config_file).expect("Unable to open config file")
    }

    fn new() -> Config {
        let mut serialized = String::new();
        Config::get_config_file().read_to_string(&mut serialized).expect("Unable to read config file");
        Config::from_serialized(&serialized)
    }

    fn from_serialized(serialized: &str) -> Config {
        serde_yaml::from_str::<Config>(serialized).expect("Unable to unserialize config file")
    }

    fn to_serialized(self: &Self) -> String {
        serde_yaml::to_string(self).expect("Unable to serialize config file")
    }
}

fn recreate_game_state() -> () {
    let mut game_state = GameState::default();
    {
        let nb_teams = 8;
        let team_colors = vec![ "#aa0000", "#00aa00", "#0000aa", "#aaaa00", "#00aaaa", "#aaaaaa", "#aa00aa", "#44aaff" ];
        let team_names  = vec![ "team 1", "team 2", "team 3", "team 4", "team 5", "team 6", "team 7", "team 8" ];
        let team_tokens  = vec![ "XXX1", "XXX2", "XXX3", "XXX4", "XXX5", "XXX6", "XXX7", "XXX8" ];
        game_state.id = 0;
        game_state.tick = 0;
        let map = &mut game_state.map;
        let teams = &mut game_state.teams;
        for i in 0..nb_teams {
            let car = Car {
                x: 0,
                y: 0,
                health: 3,
                resources: 0,
                state: State::STOPPED,
            };
            teams.push(Team {
                id: i+1,
                name: team_names[i as usize].to_string(),
                color: team_colors[i as usize].to_string(),
                score: 0,
                token: team_tokens[i as usize].to_string(),
                car: car,
            });
        }
        let level = vec![
            "WWWWWWWWWWWWWWWWWWWW",
            "W  9               W",
            "W WWWWWWWWW WWWWWW W",
            "W W              W W",
            "W W      5       W W",
            "W W             3W W",
            "W W  WWWWWWWWWWWWW W",
            "W W              1 W",
            "W W                W",
            "W WWWWWW      W    W",
            "W             W    W",
            "W      W      W    W",
            "W      W  1        W",
            "W      W           W",
            "W      WWWWWW      W",
            "W                  W",
            "W                  W",
            "W       WWWW       W",
            "W                  W",
            "W                  W",
            "WBBBBBBBBBBBBBBBBBBW",
            "WWWWWWWWWWWWWWWWWWWW",
        ];
        for level_row in level.iter() {
            let mut row: Vec<Cell> = Vec::new();
            for c in level_row.chars() {
                let block = match c {
                    'W' => Block::WALL,
                    _ => Block::OPEN
                };
                let mut items: Vec<Item> = Vec::new();
                match c {
                    'B' => items.push(Item::BASE),
                     val @ '1' ... '9' => {
                        let value = val.to_digit(10).unwrap();
                        let cooldown = 10 + value*2;
                        let producer = Producer { resource: Resource::GAS(value), cooldown: cooldown };
                        items.push(Item::PRODUCER(producer));
                    },
                    _ => ()
                };
                let cell = Cell { block: block, items: items };
                row.push(cell);
            }
            map.cells.push(row);
        }
    }
    println!("{}", game_state.to_serialized());
}


fn recreate_config() -> () {
    let mut config = Config::default();
    {
        let flags = &mut config.flags;
        for _ in 0..3 {
            flags.push(Flag { score: 1000, flag: "ICON{XXXXXXXXXX}".to_string() });
        }
    }
    println!("{}", config.to_serialized());
}


fn run(matches: ArgMatches) -> Result<(), String> {
    let log_level = match matches.occurrences_of("verbose") {
        0 => slog::Level::Info,
        1 => slog::Level::Debug,
        2 | _ => slog::Level::Trace,
    };
    init_logger(log_level);
    trace!(logger!(), "Logger initialized");

    match matches.value_of("recreate") {
        Some("game_state") => {
            recreate_game_state();
            return Ok(());
        },
        Some("config") => {
            recreate_config();
            return Ok(());
        },
        Some(param) => {
            error!(logger!(), "Invalid parameter to --recreate option: {}", param);
            return Ok(());
        },
        _ => ()
    }

    let config = Config::new();
    trace!(logger!(), "Configuration loaded");

    let mut game_engine = GameEngine::new();
    info!(logger!(), "Game initialized");

    start_ws_server();
    start_rpc_server(&config);
    info!(logger!(), "Ready to accept connections");

    game_engine.start();
    warn!(logger!(), "Game loop ended");

    Ok(())
}

fn init_logger(log_level: slog::Level) {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog::LevelFilter::new(drain, log_level).fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = slog::Logger::root(drain, o!());
    LOGGER.lock().expect("Unable to acquire LOGGER lock in init_logger()").push(logger);
}

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .multiple(true)
            .help("verbosity level"))
        .arg(Arg::with_name("recreate")
             .short("r")
             .long("recreate")
             .value_name("FILE")
             .takes_value(true)
             .help("generate an initial version of the game state or the config file, depending on the value of this parameter: 'game_state' or 'config'"))
        .get_matches();
    if let Err(_) = run(matches) {
        process::exit(1);
    }
    thread::sleep(time::Duration::from_millis(500));
}
