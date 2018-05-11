#![feature(proc_macro, generators)]

extern crate futures_await as futures;
extern crate tokio;
extern crate tokio_serialize_stream;
#[macro_use]
extern crate serde_derive;

mod common;

use futures::prelude::*;
use tokio_serialize_stream::MsgListener;

use common::Message;

#[async]
fn server() -> Result<(), tokio_serialize_stream::Error> {
    let addr = "127.0.0.1:5000".parse().unwrap();
    let listener = MsgListener::bind(&addr)?;
    #[async]
    for client in listener.incoming() {
        let peer_addr = client.peer_addr().expect("unable to get peer address");
        println!("New client connection from {:?}", peer_addr);
        let client = await!(client.send(Message::Hello(format!("{}", peer_addr))))?;
        await!(client.send(Message::GoodBye))?;
    }
    Ok(())
}

fn main() {
    tokio::run(server().map_err(|err| eprintln!("error {:?}", err)));
}
