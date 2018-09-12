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

extern crate futures;
#[macro_use]
extern crate tarpc;
extern crate tokio_core;

#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

use std::process;
use std::thread;
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

mod websockets;
use websockets::start_ws_server;


fn run(matches: ArgMatches) -> Result<(), String> {
    let log_level = match matches.occurrences_of("verbose") {
        0 => slog::Level::Info,
        1 => slog::Level::Debug,
        2 | _ => slog::Level::Trace,
    };
    let simulate = matches.is_present("simulate");

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

    let mut game_engine = GameEngine::new(simulate);
    info!(logger!(), "Game initialized");

    let ws_broadcaster = start_ws_server();
    start_rpc_server(config);
    info!(logger!(), "Ready to accept connections");

    game_engine.start(ws_broadcaster);
    warn!(logger!(), "Game loop ended");

    Ok(())
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
