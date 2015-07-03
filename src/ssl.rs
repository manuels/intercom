use std::sync::{Arc,Mutex};
use std::io;
use std::io::{Read,Write};
use std::sync::mpsc::{Sender,Receiver};
use std::sync::mpsc::channel;
use std::thread;
use libc::consts::os::bsd44::SOCK_DGRAM;

use openssl::ssl::{SslStream, SslContext};
use openssl::ssl::error::SslError;

use std::os::unix::io::AsRawFd;

use utils::is_readable::IsReadable;
use utils::socket::ChannelToSocket;
use nonblocking_socket::NonBlockingSocket;

use syscalls;

pub struct SslChannel
{
	stream: Arc<Mutex<SslStream<NonBlockingSocket<ChannelToSocket>>>>,
	is_server: bool,
}

impl SslChannel
{
	pub fn new(ctx: &SslContext, is_server: bool,
	           ciphertext_ch: (Sender<Vec<u8>>,Receiver<Vec<u8>>),
	           plaintext_ch:  (Sender<Vec<u8>>,Receiver<Vec<u8>>))
		-> Result<SslChannel,SslError>
	{
		let (ciphertext_tx, ciphertext_rx) = ciphertext_ch;
		let (ciphertext_rx, is_readable) = IsReadable::new(ciphertext_rx);
		
		let ciphertext = ChannelToSocket::new_from(SOCK_DGRAM, 0, 
			(ciphertext_tx, ciphertext_rx))
			.ok().expect("Creating ChannelToSocket failed");

		let (plaintext_tx,  plaintext_rx) = plaintext_ch;

		debug!("{} SSL pre handshake 1/2", is_server);

		let ciphertext_fd = ciphertext.as_raw_fd();
		let ciphertext_rw = NonBlockingSocket::new(ciphertext);

		let stream = match is_server {
			true  => try!(SslStream::new_server(ctx, ciphertext_rw)),
			false => try!(SslStream::new(ctx, ciphertext_rw)),
		};

		info!("{} SSL handshake done! 2/2", is_server);

		let channel = SslChannel {
			stream: Arc::new(Mutex::new(stream)),
			is_server: is_server,
		};
		syscalls::set_blocking(ciphertext_fd, false).unwrap();

		channel.spawn_read(plaintext_tx, is_readable).unwrap();
		channel.spawn_write(plaintext_rx).unwrap();

		Ok(channel)
	}

	fn spawn_write(&self, plaintext_rx:  Receiver<Vec<u8>>)
		-> io::Result<thread::JoinHandle<()>>
	{
		let stream = self.stream.clone();
		let is_server = self.is_server;

		thread::Builder::new().name("SslChannel::spawn_write".to_string()).spawn(move || {
			for buf in plaintext_rx.iter() {
				let mut s = stream.lock().unwrap();

				let len = (*s).write(&buf[..]).unwrap();
				(*s).flush().unwrap();

				debug!("{} plaintext_rx SSL_written len={}, buf.len={}", is_server, len, buf.len());
				assert_eq!(len, buf.len())
			}
			panic!("fin");
		})
	}

	fn spawn_read(&self,
	              plaintext_tx: Sender<Vec<u8>>,
	              is_readable:  IsReadable)
		-> io::Result<thread::JoinHandle<()>>
	{
		let stream = self.stream.clone();

		let is_readable = is_readable.unpack();
		thread::Builder::new().name("SslChannel::spawn_read".to_string()).spawn(move || {
			loop {
				let mut buf = vec![0; 16*1024];

				let &(ref lock, ref cvar) = &*is_readable;

				let mut readable = lock.lock().unwrap();
				let mut s = stream.lock().unwrap();
				while !*readable && (*s).pending() == 0 {
					drop(s);
					readable = cvar.wait(readable).unwrap();
					s = stream.lock().unwrap();
				}
				drop(s);

				let mut s = stream.lock().unwrap();

				let len = s.read(&mut buf[..]).unwrap();

				if len > 0 {
					buf.truncate(len);

					if plaintext_tx.send(buf.clone()).is_err() {
						break
					}
				}

				*readable = false;
			}
		})
	}
}
