extern crate bincode;
extern crate bytes;
#[macro_use]
extern crate futures;
extern crate serde;
extern crate tokio;
extern crate tokio_io;

use std::io::{self, Cursor};
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::ops::{Deref, DerefMut};

use bytes::{Buf, BufMut, BytesMut};
use futures::{Async, Poll};
use tokio_io::codec;
use tokio_io::AsyncRead;

const HEADER_SIZE: usize = 4;

struct SimpleCodec<T> {
    _marker: PhantomData<T>,
}

impl<T> SimpleCodec<T> {
    fn new() -> Self {
        SimpleCodec {
            _marker: PhantomData,
        }
    }
}

impl<T> codec::Decoder for SimpleCodec<T>
where
    T: serde::de::DeserializeOwned,
{
    type Item = T;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() >= HEADER_SIZE {
            let msg_size = {
                let mut cursor = Cursor::new(&src);
                cursor.get_u32_be() as usize
            };

            // println!("msg {} buf size {}", msg_size, src.len());

            if src[HEADER_SIZE..].len() >= msg_size {
                src.advance(HEADER_SIZE); // skip header, must not deserialize
                let buf = src.split_to(msg_size);
                // println!("split buf size {}", buf.len());
                return Ok(Some(bincode::deserialize(&buf)?));
            }
        }
        Ok(None)
    }
}

impl<T> codec::Encoder for SimpleCodec<T>
where
    T: serde::Serialize,
{
    type Item = T;
    type Error = Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let msg_size = bincode::serialized_size(&item)?;
        dst.put_u32_be(msg_size as u32);
        bincode::serialize_into(dst.writer(), &item)?;
        Ok(())
    }
}

pub struct MsgStream<T> {
    inner: codec::Framed<tokio::net::TcpStream, SimpleCodec<T>>,
}

impl<T> MsgStream<T> {
    pub fn connect(addr: &SocketAddr) -> ConnectFuture<T> {
        ConnectFuture {
            inner: tokio::net::TcpStream::connect(addr),
            _marker: PhantomData,
        }
    }
}

pub struct ConnectFuture<T> {
    inner: tokio::net::ConnectFuture,
    _marker: PhantomData<T>,
}

impl<T> futures::Future for ConnectFuture<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    type Item = MsgStream<T>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let sock = try_ready!(self.inner.poll());
        Ok(Async::Ready(MsgStream {
            inner: sock.framed(SimpleCodec::new()),
        }))
    }
}

impl<T> futures::Stream for MsgStream<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    type Item = T;
    type Error = Error;

    #[inline]
    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        futures::Stream::poll(&mut self.inner)
    }
}

impl<T> futures::Sink for MsgStream<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    type SinkItem = T;
    type SinkError = Error;

    #[inline]
    fn start_send(
        &mut self,
        item: Self::SinkItem,
    ) -> futures::StartSend<Self::SinkItem, Self::SinkError> {
        futures::Sink::start_send(&mut self.inner, item)
    }

    #[inline]
    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        futures::Sink::poll_complete(&mut self.inner)
    }

    #[inline]
    fn close(&mut self) -> Poll<(), Self::SinkError> {
        futures::Sink::close(&mut self.inner)
    }
}

impl<T> Deref for MsgStream<T> {
    type Target = tokio::net::TcpStream;

    fn deref(&self) -> &Self::Target {
        self.inner.get_ref()
    }
}

impl<T> DerefMut for MsgStream<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.get_mut()
    }
}

pub struct MsgListener<T> {
    inner: tokio::net::TcpListener,
    _marker: PhantomData<T>,
}

impl<T> MsgListener<T> {
    pub fn bind(addr: &SocketAddr) -> io::Result<MsgListener<T>> {
        let sock = tokio::net::TcpListener::bind(addr)?;
        Ok(MsgListener {
            inner: sock,
            _marker: PhantomData,
        })
    }
}

impl<T> MsgListener<T> {
    pub fn incoming(self) -> Incoming<T> {
        Incoming {
            inner: self.inner.incoming(),
            _marker: PhantomData,
        }
    }
}

impl<T> Deref for MsgListener<T> {
    type Target = tokio::net::TcpListener;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for MsgListener<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct Incoming<T> {
    inner: tokio::net::Incoming,
    _marker: PhantomData<T>,
}

impl<T> futures::Stream for Incoming<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    type Item = MsgStream<T>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let sock = try_ready!(self.inner.poll());
        if let Some(sock) = sock {
            let stream = MsgStream {
                inner: sock.framed(SimpleCodec::new()),
            };
            return Ok(Async::Ready(Some(stream)));
        }
        Ok(Async::Ready(None))
    }
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Serialization(bincode::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<bincode::Error> for Error {
    fn from(err: bincode::Error) -> Self {
        Error::Serialization(err)
    }
}
