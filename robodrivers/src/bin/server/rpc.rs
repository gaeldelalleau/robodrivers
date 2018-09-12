use std::collections::HashMap;
use std::thread;

use tarpc::sync::server;
use tarpc::util::{FirstSocketAddr, Message, Never};

use config::Config;
use config::TeamConfig;
use logging::LOGGER;
use game_engine::game_state_map;
use game_engine::actions_map;
use game_state::Team;

use robodrivers::Action;
use robodrivers::rpc;
use ::robodrivers::rpc::SyncServiceExt;


const MAX_RPC_REQUEST_SIZE: u64 = 8192; // in bytes


struct TeamInfo {
    score: u32,
}

fn check_authentication(team_id: u32, token: String, teams_config: &HashMap<u32, TeamConfig>, teams: &HashMap<u32, Team>) -> Result<TeamInfo, String> {
    match teams_config.get(&team_id) {
        Some(team_config) =>
            if token == team_config.token {
                match teams.get(&team_id) {
                    Some(team) => return Ok(TeamInfo { score: team.score }),
                    None => (),
                }
            },
        None => (),
    }
    debug!(logger!(), "Received invalid team_id or token, forbidding RPC request");
    return Err("Invalid team_id or token".to_string());
}

fn is_authenticated(team_id: u32, token: String, config: &Config) -> Result<TeamInfo, String> {
    let mut game_state_guard = game_state_guard!();
    let game_state = game_state!(game_state_guard);
    check_authentication(team_id, token, &config.teams_config, &game_state.teams)
}

fn check_tick(game_tick: u32, tick: u32) -> Result<(), String> {
    if game_tick != tick {
        debug!(logger!(), "Received invalid tick {} instead of {}, forbidding RPC request", tick, game_tick);
        return Err(format!("Current tick is {}, but received action for tick {}", game_tick, tick));
    }
    Ok(())
}

#[derive(Clone)]
struct CommandsServer {
    config: Config,
}

impl rpc::SyncService for CommandsServer {

    fn action(&self, team_id: u32, token: String, action: Action, tick: u32) -> Result<String, Message> {
        trace!(logger!(), "Received RPC action: team {}, token {}, action {:?}, tick {}", team_id, token, action, tick);
        match is_authenticated(team_id, token, &self.config) {
            Err(err) => return Err(Message(err)),
            Ok(_) => (),
        }
        {
            let mut game_state_guard = game_state_guard!();
            match check_tick(game_state!(game_state_guard).tick, tick) {
                Err(err) => return Err(Message(err)),
                Ok(_) => (),
            }
            debug!(logger!(), "Registering valid action {:?} for team id {} at tick {}", action, team_id, tick);
            actions!().insert(team_id, action);
        }
        Ok(format!("Ok"))
    }

    fn flags(&self, team_id: u32, token: String) -> Result<String, Message> {
        trace!(logger!(), "Received RPC flags: team id {}, token {}", team_id, token);
        let team: TeamInfo;
        match is_authenticated(team_id, token, &self.config) {
            Err(err) => return Err(Message(err)),
            Ok(t) => team = t,
        }
        debug!(logger!(), "Received valid flag request for team id {}", team_id);

        let flags: Vec<String> = self.config.flags.iter().map(
            |f| if f.score <= team.score {
                f.flag.clone()
            } else {
                format!("Flag LOCKED until you reach score {}", f.score)
            }).collect();
        Ok(format!("Your team unlocked those flags, make sure to submit them in the CTF submission interface: {:?}", flags))
    }

    fn ping(&self) -> Result<String, Never> {
        trace!(logger!(), "Received RPC ping");
        Ok(format!("pong"))
    }
}

pub fn start_rpc_server(config: Config) {
    trace!(logger!(), "Starting RPC server");

    let mut options = server::Options::default();
    options = options.max_payload_size(MAX_RPC_REQUEST_SIZE);

    let _rpc_server = thread::Builder::new().name("rpc_server".to_owned()).spawn(move || {
        let handler = CommandsServer { config: config }.listen("0:3011".first_socket_addr(), options).expect("Unable to listen on socket for RPC server");
        handler.run();
    }).expect("Unable to spawn new thread for RPC server");

    trace!(logger!(), "RPC server started");
}
