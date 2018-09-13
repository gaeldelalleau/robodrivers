#![feature(plugin)]
#![plugin(tarpc_plugins)]

extern crate robodrivers;

#[macro_use]
extern crate lazy_static;
extern crate dirs;
extern crate clap;
extern crate ws;
extern crate rand;

#[macro_use]
extern crate slog;
extern crate slog_term;
extern crate slog_async;

extern crate tarpc;

#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

use std::process;
use std::thread;
use std::sync::mpsc;
use std::time;
use clap::{Arg, ArgMatches, App};

#[macro_use]
mod logging;
use logging::init_logger;
use logging::LOGGER;

mod game_state;
use game_state::recreate_game_state;

#[macro_use]
mod game_engine;
use game_engine::GameEngine;

mod config;
use config::Config;
use config::recreate_config;

mod rpc;
use rpc::start_rpc_server;
use rpc::Command;

mod websockets;
use websockets::start_ws_server;


fn run(matches: ArgMatches) -> Result<(), String> {
    let log_level = match matches.occurrences_of("verbose") {
        0 => slog::Level::Info,
        1 => slog::Level::Debug,
        2 | _ => slog::Level::Trace,
    };
    let simulate = matches.is_present("simulate");
    let remote_control = matches.is_present("remote_control");

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

    let config_dir = matches.value_of("config_dir");

    let (config, config_dir_path) = Config::new(config_dir);
    trace!(logger!(), "Configuration loaded");

    let (send, recv) = mpsc::channel();

    let ws_broadcaster = start_ws_server();
    start_rpc_server(send, config, remote_control);
    info!(logger!(), "Ready to accept connections");


    let mut game_engine = GameEngine::new(simulate, &config_dir_path, remote_control);
    info!(logger!(), "Game engine created");

    game_engine.init();
    info!(logger!(), "Game engine initialized");

    if remote_control {
        info!(logger!(), "Starting remote-controlled game loop");
        loop {
            let command: Command = recv.recv().expect("Channel error while receiving commands from the RPC thread");
            match command {
                Command::STEP => {
                    trace!(logger!(), "One step forward...");
                    game_engine.step();
                    game_engine.broadcast(&ws_broadcaster);
                },
                Command::RESET => {
                    info!(logger!(), "Resetting game state and engine");
                    game_engine = GameEngine::new(simulate, &config_dir_path, true);
                    game_engine.init();
                },
            }

        }

    } else {
        info!(logger!(), "Starting automatic game loop");
        loop {
            thread::sleep(time::Duration::from_millis(100));
            game_engine.step();
            game_engine.broadcast(&ws_broadcaster);
            game_engine.save_periodically(10);
        }
    }
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
        .arg(Arg::with_name("simulate")
            .short("s")
            .long("simulate")
            .help("simulate team actions"))
        .arg(Arg::with_name("remote_control")
            .short("z")
            .long("remote_control")
            .help("activate control of the game engine (step and reset RPC actions performed by the client, no automatic steps) instead of running the simulation automatically"))
        .arg(Arg::with_name("recreate")
             .short("r")
             .long("recreate")
             .value_name("FILE")
             .takes_value(true)
             .help("generate an initial version of the game state or the config file, depending on the value of this parameter: 'game_state' or 'config'"))
        .arg(Arg::with_name("config_dir")
             .short("c")
             .long("config_dir")
             .value_name("CONFIG_DIR")
             .takes_value(true)
             .help("Specify the path to the folder holding the config.yaml and base_game_state.yaml files"))
        .get_matches();
    if let Err(_) = run(matches) {
        process::exit(1);
    }
    thread::sleep(time::Duration::from_millis(500));  // hack to allow the shared async logger some time to flush its output
}
