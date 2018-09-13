#![crate_type="lib"]

#![feature(plugin)]
#![plugin(tarpc_plugins)]

extern crate rand;

#[macro_use]
extern crate tarpc;
extern crate tokio_core;
extern crate futures;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

pub mod actions;
pub use actions::Direction;
pub use actions::Action;

pub mod rpc;
pub use rpc::rpc_flags;
pub use rpc::rpc_action;
pub use rpc::rpc_step;
pub use rpc::rpc_reset;
pub use rpc::free_string;
