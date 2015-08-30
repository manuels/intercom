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
use connection::ControllingMode;
use connection::ControllingMode::{Server, Client};

use syscalls;

pub struct SslChannel
{
	stream:           Arc<Mutex<SslStream<NonBlockingSocket<ChannelToSocket>>>>,
	controlling_mode: ControllingMode,
}

impl SslChannel
{
	pub fn new(ctx:              &SslContext,
	           controlling_mode: ControllingMode,
	           ciphertext_ch:    (Sender<Vec<u8>>,Receiver<Vec<u8>>),
	           plaintext_ch:     (Sender<Vec<u8>>,Receiver<Vec<u8>>))
		-> Result<SslChannel,SslError>
	{
		let (ciphertext_tx, ciphertext_rx) = ciphertext_ch;
		let (ciphertext_rx, is_readable) = IsReadable::new(ciphertext_rx);
		
		let ciphertext = ChannelToSocket::new_from(SOCK_DGRAM, 0, 
			(ciphertext_tx, ciphertext_rx))
			.ok().expect("Creating ChannelToSocket failed");

		let (plaintext_tx,  plaintext_rx) = plaintext_ch;

		debug!("{:?} SSL pre handshake 1/2", controlling_mode);

		let ciphertext_fd = ciphertext.as_raw_fd();
		let ciphertext_rw = NonBlockingSocket::new(ciphertext);

		let stream = match controlling_mode {
			Server => try!(SslStream::accept_generic(ctx, ciphertext_rw)),
			Client => try!(SslStream::connect_generic(ctx, ciphertext_rw)),
		};

		info!("{:?} SSL handshake done! 2/2", controlling_mode);

		let channel = SslChannel {
			stream:           Arc::new(Mutex::new(stream)),
			controlling_mode: controlling_mode,
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
		let controlling_mode = self.controlling_mode;

		thread::Builder::new().name("SslChannel::spawn_write".to_string()).spawn(move || {
			for buf in plaintext_rx.iter() {
				let mut s = stream.lock().unwrap();

				let len = (*s).write(&buf[..]).unwrap();
				(*s).flush().unwrap();

				debug!("{:?} plaintext_rx SSL_written len={}, buf.len={}", controlling_mode, len, buf.len());
				assert_eq!(len, buf.len())
			}
			debug!("SslChannel::spawn_write");
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

				debug!("reading...");
				let res = s.read(&mut buf[..]);
				debug!("read {:?}", res);

				match res {
					Ok(len) if len == 0 => (), //continue,
					Ok(len) => {
						buf.truncate(len);

						if plaintext_tx.send(buf.clone()).is_err() {
							break
						}
					},
					Err(err) => {
						warn!("{:?}", err);
						break;
					}
				}

				*readable = false;
			}
			info!("SslChannel::spawn_read: read loop ended.");
		})
	}
}
