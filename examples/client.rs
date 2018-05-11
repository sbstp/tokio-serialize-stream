extern crate futures;
extern crate tokio;
extern crate tokio_serialize_stream;
#[macro_use]
extern crate serde_derive;

mod common;

use futures::{Future, Sink, Stream};
use tokio_serialize_stream::MsgStream;

use common::Message;

fn main() {
    let addr = "127.0.0.1:5000".parse().unwrap();
    let fut = MsgStream::connect(&addr)
        .and_then(|msg_stream: MsgStream<Message>| {
            msg_stream.for_each(|msg| {
                println!("received message {:?}", msg);
                Ok(())
            })
        })
        .map_err(|err| println!("error {:?}", err));
    tokio::run(fut);
}
