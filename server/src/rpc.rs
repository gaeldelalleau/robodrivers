use tarpc::sync::client;
use tarpc::sync::client::ClientExt;

use tarpc::util::Message;
use tarpc::util::Never;

use std::net::SocketAddr;

use std::ffi::{CString, CStr};
use std::os::raw::c_char;

use std::str::Utf8Error;

use Action;


service! {
    rpc action(team_id: u32, token: String, action: Action, tick: u32) -> String | Message;
    rpc flags(team_id: u32, token: String) -> String | Message;
    rpc step(team_id: u32, token: String) -> String | Message;
    rpc reset(team_id: u32, token: String) -> String | Message;
    rpc ping() -> String | Never;
}


fn str_from<'a>(cstring: *const c_char) -> Result<&'a str, Utf8Error> {
    unsafe {
        assert!(!cstring.is_null());
        CStr::from_ptr(cstring)
    }.to_str()
}

fn c_char_from(string: String) -> *mut c_char {
    CString::new(string).expect("CString conversion error").into_raw()
}

fn convert_params<'a>(cstring_host_and_port: *const c_char, cstring_token: *const c_char) -> Result<(&'a str, &'a str, SocketAddr), *mut c_char> {
    let host_and_port = match str_from(cstring_host_and_port) {
        Ok(s) => s,
        Err(e) => return Err(c_char_from(format!("ERROR: CString conversion error for variable cstring_host_and_port: {}", e))),
    };
    let token = match str_from(cstring_token) {
        Ok(s) => s,
        Err(e) => return Err(c_char_from(format!("ERROR: CString conversion error for variable cstring_token: {}", e))),
    };
    let socket_addr: SocketAddr = match host_and_port.parse() {
        Ok(s) => s,
        Err(e) => return Err(c_char_from(format!("ERROR: Unable to parse socket address from {}: {}", host_and_port, e))),
    };
    Ok((host_and_port, token, socket_addr))
}

fn rpc_perform<C>(command: C, cstring_host_and_port: *const c_char, cstring_token: *const c_char) -> *mut c_char
    where C: FnOnce(SyncClient, String) -> Result<String, ::tarpc::Error<Message>> {

    let (host_and_port, token, socket_addr) = match convert_params(cstring_host_and_port, cstring_token) {
        Ok((h, t, s)) => (h, t, s),
        Err(e) => return e,
    };

    let options = client::Options::default();
    let client = match SyncClient::connect(socket_addr, options) {
        Ok(c) => c,
        Err(e) => return c_char_from(format!("ERROR: Unable to connect to RPC service on {}: {}", host_and_port, e)),
    };

    let response = match command(client, token.to_string()) {
        Ok(r) => r,
        Err(e) => return c_char_from(format!("ERROR: error while doing RPC communication: {}", e)),
    };

    c_char_from(format!("{}", response))
}

fn parse_action(cstring_action: *const c_char) -> Result<Action, String> {
    let action = match str_from(cstring_action) {
        Ok(s) => s,
        Err(e) => return Err(format!("ERROR: CString conversion error for variable cstring_action: {}", e)),
    };
    match ::serde_json::from_str(action) {
        Ok(a) => Ok(a),
        Err(e) => Err(format!("{}", e)),
    }
}

#[no_mangle]
pub extern fn rpc_action(cstring_host_and_port: *const c_char, team_id: u32, cstring_token: *const c_char, cstring_action: *const c_char, tick: u32) -> *mut c_char {
    let action = match parse_action(cstring_action) {
        Ok(a) => a,
        Err(e) => return c_char_from(format!("ERROR: action not recognized : {}", e)),
    };
    rpc_perform(|client, token| { client.action(team_id, token, action, tick) }, cstring_host_and_port, cstring_token)
}

#[no_mangle]
pub extern fn rpc_flags(cstring_host_and_port: *const c_char, team_id: u32, cstring_token: *const c_char) -> *mut c_char {
    rpc_perform(|client, token| { client.flags(team_id, token) }, cstring_host_and_port, cstring_token)
}

#[no_mangle]
pub extern fn rpc_step(cstring_host_and_port: *const c_char, team_id: u32, cstring_token: *const c_char) -> *mut c_char {
    rpc_perform(|client, token| { client.step(team_id, token) }, cstring_host_and_port, cstring_token)
}

#[no_mangle]
pub extern fn rpc_reset(cstring_host_and_port: *const c_char, team_id: u32, cstring_token: *const c_char) -> *mut c_char {
    rpc_perform(|client, token| { client.reset(team_id, token) }, cstring_host_and_port, cstring_token)
}

#[no_mangle]
pub extern fn rpc_ping(cstring_host_and_port: *const c_char) -> *mut c_char {
    let host_and_port = match str_from(cstring_host_and_port) {
        Ok(s) => s,
        Err(e) => return c_char_from(format!("ERROR: CString conversion error for variable cstring_host_and_port: {}", e)),
    };

    let socket_addr: SocketAddr = match host_and_port.parse() {
        Ok(s) => s,
        Err(e) => return c_char_from(format!("ERROR: Unable to parse socket address from {}: {}", host_and_port, e)),
    };

    let options = client::Options::default();
    let client = match SyncClient::connect(socket_addr, options) {
        Ok(c) => c,
        Err(e) => return c_char_from(format!("ERROR: Unable to connect to RPC service on {}: {}", host_and_port, e)),
    };

    let response = match client.ping() {
        Ok(r) => r,
        Err(e) => return c_char_from(format!("ERROR: error while doing RPC communication: {}", e)),
    };

    c_char_from(format!("{}", response))
}

#[no_mangle]
pub extern fn free_string(string: *mut c_char) {
    unsafe { CString::from_raw(string); }
}
