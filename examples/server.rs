extern crate futures;
extern crate tokio;
extern crate tokio_serialize_stream;
#[macro_use]
extern crate serde_derive;

mod common;

use futures::{Future, Sink, Stream};
use tokio_serialize_stream::MsgListener;

use common::Message;

fn main() {
    let addr = "127.0.0.1:5000".parse().unwrap();
    let listener = MsgListener::bind(&addr).unwrap();
    let fut = listener
        .incoming()
        .for_each(|sock| {
            println!("new client {:?}", sock.peer_addr());
            sock.send(Message::Hello).and_then(|_| Ok(()))
        })
        .map_err(|err| println!("error {:?}", err));
    tokio::run(fut);
}
