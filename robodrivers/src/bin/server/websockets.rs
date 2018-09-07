use std::thread;
use ws;
use ws::{listen, Handler, Sender, Message, CloseCode};
use logging::LOGGER;


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

pub fn start_ws_server() -> () {
    trace!(logger!(), "Starting WebSocket server");
    let _ws_server = thread::Builder::new().name("ws_server".to_owned()).spawn(move || {
        listen("0:3012", |out| Server { out: out } ).expect("Unable to listen on socket for Websocket server");
    }).expect("Unable to spawn new thread for Websocket server");
    trace!(logger!(), "WebSocket server started");
}

