use std::thread;
use ws;
use ws::{Handler, Sender, Message, CloseCode, Error, Handshake};
use logging::LOGGER;


struct Server {
    out: Sender,
    client_addr: String,
}

impl Handler for Server {

    fn on_open(&mut self, shake: Handshake) -> ws::Result<()> {
        let request = &shake.request;
        let origin = request.origin()
            .unwrap_or(Some("<no origin found>")).unwrap_or("<no origin found>");  // Result<Option<&str>>
        let client_addr = &shake.remote_addr()
            .unwrap_or(Some("<no client ip found>".to_string())).unwrap_or("<no client ip found>".to_string());  // Result<Option<String>>
        debug!(logger!(), "WebSocket connection received from ip {} by origin {}", client_addr, origin);
        self.client_addr = client_addr.clone();
        Ok(())
    }

    fn on_message(&mut self, _msg: Message) -> ws::Result<()> {
        self.out.send("wut?")
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        match code {
            CloseCode::Normal => debug!(logger!(), "The client {} is done with the connection.", self.client_addr),
            CloseCode::Away => debug!(logger!(), "The client {} is leaving the site.", self.client_addr),
            CloseCode::Abnormal => info!(logger!(), "Closing handshake failed! Unable to obtain closing status from client {}.", self.client_addr),
            _ => info!(logger!(), "The client {} encountered an error: {}", self.client_addr, reason),
        }
    }

    fn on_error(&mut self, err: Error) {
        error!(logger!(), "The websocket server encountered an error: {:?}", err);
    }
}

pub fn start_ws_server() -> Sender {
    trace!(logger!(), "Starting WebSocket server");

    let socket = ws::WebSocket::new(|out| Server { out: out, client_addr: "unknown".to_string() }).expect("Unable to create new WebSocket instance");
    let broadcaster = socket.broadcaster();
    let _ws_server = thread::Builder::new().name("ws_server".to_owned()).spawn(move || {
        socket.listen("0:3012").expect("Unable to listen on socket for Websocket server");
    }).expect("Unable to spawn new thread for Websocket server");

    trace!(logger!(), "WebSocket server started");
    broadcaster
}
