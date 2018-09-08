use std::thread;
use ws;
use ws::{Handler, Sender, Message, CloseCode, Error};
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
            CloseCode::Away => debug!(logger!(), "The client is leaving the site."),
            CloseCode::Abnormal => info!(logger!(), "Closing handshake failed! Unable to obtain closing status from client."),
            _ => info!(logger!(), "The client encountered an error: {}", reason),
        }
    }

    fn on_error(&mut self, err: Error) {
        error!(logger!(), "The websocket server encountered an error: {:?}", err);
    }
}

pub fn start_ws_server() -> Sender {
    trace!(logger!(), "Starting WebSocket server");

    let socket = ws::WebSocket::new(|out| Server { out: out }).expect("Unable to create new WebSocket instance");
    let broadcaster = socket.broadcaster();
    let _ws_server = thread::Builder::new().name("ws_server".to_owned()).spawn(move || {
        socket.listen("0:3012").expect("Unable to listen on socket for Websocket server");
    }).expect("Unable to spawn new thread for Websocket server");

    trace!(logger!(), "WebSocket server started");
    broadcaster
}
