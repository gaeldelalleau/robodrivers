use tarpc::future::server;
use tarpc::util::{FirstSocketAddr, Never}; 
use tokio_core::reactor;

use config::Config;
use logging::LOGGER;

const MAX_RPC_REQUEST_SIZE: u64 = 8192; // in bytes


pub mod commands {
    service! {
        rpc action(name: String) -> String;
        rpc flags() -> String;
    }
}
use rpc::commands::FutureServiceExt as CommandsExt;

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

pub fn start_rpc_server(_config: &Config) -> () {
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

