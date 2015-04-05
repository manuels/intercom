use std::sync::{Arc,Mutex};
use std::sync::Condvar;
use std::io::{Read,Write};
use std::sync::mpsc::{Sender,Receiver};
use std::sync::mpsc::channel;
use std::thread;

use openssl::ssl::{SslStream, SslContext};
use openssl::ssl::error::SslError;

use utils::pipe::{Pipe,ChannelToReadWrite};

pub struct SslChannel
{
	stream: Arc<Mutex<SslStream<ChannelToReadWrite>>>
}

impl SslChannel
{
	pub fn new(ctx: &SslContext, is_server: bool,
	           ciphertext_ch: (Sender<Vec<u8>>,Receiver<Vec<u8>>),
	           plaintext_ch:  (Sender<Vec<u8>>,Receiver<Vec<u8>>))
		-> Result<SslChannel,SslError>
	{
		let (ciphertext_tx, ciphertext_rx) = ciphertext_ch;
		let (plaintext_tx,  plaintext_rx)  = plaintext_ch;

		let ciphertext_pipe = ChannelToReadWrite::new(ciphertext_tx, ciphertext_rx);
		let is_readable = ciphertext_pipe.is_readable();

		info!("SSL pre handshake");
		let stream = match is_server {
			true => try!(SslStream::new_server(&ctx, ciphertext_pipe)),
			false => try!(SslStream::new(&ctx, ciphertext_pipe)),
		};
		info!("SSL handshake done!");
		
		let channel = SslChannel {
			stream: Arc::new(Mutex::new(stream))
		};

		channel.spawn_read(is_readable, plaintext_tx);
		channel.spawn_write(plaintext_rx);

		Ok(channel)
	}

	fn spawn_write(&self,
	               plaintext_rx:  Receiver<Vec<u8>>)
	{
		let stream = self.stream.clone();

//		spawn(move || {
		thread::Builder::new().name("SslChannel::spawn_write".to_string()).spawn(move || {
			for buf in plaintext_rx.iter() {
				let mut s = stream.lock().unwrap();
				(*s).write(buf.as_slice()); // blocking?
			}
			panic!("fin");
		});
	}

	fn spawn_read(&self,
	              is_readable:   Arc<Condvar>,
	              plaintext_tx:  Sender<Vec<u8>>)
	{
		let stream = self.stream.clone();

		thread::Builder::new().name("SslChannel::spawn_read".to_string()).spawn(move || {
			loop {
				let mut buf = vec![0; 8*1024];
				let mut s = stream.lock().unwrap();
				info!("is_readable wait");
				//let mut s = (*is_readable).wait(s).unwrap();
				info!("is_readable wait done");
				
				let len = s.read(buf.as_mut_slice()).unwrap();
				buf.truncate(len);

				plaintext_tx.send(buf).unwrap();
			}
			panic!("fin");
		});
	}
}
