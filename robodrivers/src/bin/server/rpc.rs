use std::collections::HashMap;

use tarpc::future::server;
use tarpc::util::{FirstSocketAddr};
use tarpc::util::Message;
use tokio_core::reactor;

use config::Config;
use logging::LOGGER;
use game_engine::game_state_map;
use game_engine::actions_map;
use game_engine::Action;
use game_state::Team;


const MAX_RPC_REQUEST_SIZE: u64 = 8192; // in bytes


pub mod commands {
    use rpc::Action;
    use rpc::Message;

    service! {
        rpc action(team_id: u32, token: String, action: Action, tick: u32) -> String | Message;
        rpc flags(team_id: u32, token: String) -> String | Message;
    }
}
use rpc::commands::FutureServiceExt as CommandsExt;

struct TeamInfo {
    id: u32,
    score: u32,
}

fn check_authentication(team_id: u32, token: String, teams: &HashMap<u32, Team>) -> Result<TeamInfo, String> {
    match teams.get(&team_id) {
        Some(team) =>
            if token == team.token {
                return Ok(TeamInfo { id: team_id, score: team.score });
            },
        None => (),
    }
    return Err("Invalid team_id or token".to_string());
}

fn is_authenticated(team_id: u32, token: String) -> Result<TeamInfo, String> {
    let mut game_state_guard = game_state_guard!();
    let game_state = game_state!(game_state_guard);
    check_authentication(team_id, token, &game_state.teams)
}

fn check_tick(game_tick: u32, tick: u32) -> Result<(), String> {
    if game_tick != tick {
        return Err(format!("Current tick is {}, but received action for tick {}", game_tick, tick));
    }
    Ok(())
}

#[derive(Clone)]
struct CommandsServer {
    config: Config,
}

impl commands::FutureService for CommandsServer {
    type ActionFut = Result<String, Message>;

    fn action(&self, team_id: u32, token: String, action: Action, tick: u32) -> Self::ActionFut {
        trace!(logger!(), "Received RPC action: team {}, token {}, action {:?}, tick {}", team_id, token, action, tick);
        match is_authenticated(team_id, token) {
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

    type FlagsFut = Result<String, Message>;

    fn flags(&self, team_id: u32, token: String) -> Self::FlagsFut {
        trace!(logger!(), "Received RPC flags: team id {}, token {}", team_id, token);
        let team: TeamInfo;
        match is_authenticated(team_id, token) {
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
}

pub fn start_rpc_server(config: Config) -> reactor::Core {
    trace!(logger!(), "Starting RPC server");

    let reactor = reactor::Core::new().expect("Unable to create a new Reactor");
    let mut options = server::Options::default();
    options = options.max_payload_size(MAX_RPC_REQUEST_SIZE);
    let (_handle, server) = CommandsServer { config: config }.listen("0:3011".first_socket_addr(),
                                  &reactor.handle(),
                                  options)
                          .expect("Unable to listen on socket for RPC server");
    reactor.handle().spawn(server);

    trace!(logger!(), "RPC server started");
    reactor

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
