#![feature(proc_macro, generators)]

extern crate futures_await as futures;
extern crate tokio;
extern crate tokio_serialize_stream;
#[macro_use]
extern crate serde_derive;

mod common;

use futures::prelude::*;
use tokio_serialize_stream::MsgStream;

use common::Message;

#[async]
fn client() -> Result<(), tokio_serialize_stream::Error> {
    let addr = "127.0.0.1:5000".parse().unwrap();
    let stream: MsgStream<Message> = await!(MsgStream::connect(&addr))?;
    #[async]
    for msg in stream {
        println!("recv message {:?}", msg);
    }
    Ok(())
}

fn main() {
    tokio::run(client().map_err(|err| eprintln!("error {:?}", err)));
}
