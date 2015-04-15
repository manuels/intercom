use std::sync::{Arc,Mutex};
use std::io::{Read,Write};
use std::sync::mpsc::{Sender,Receiver};
use std::sync::mpsc::channel;
use std::thread;
use libc::consts::os::bsd44::SOCK_DGRAM;
use libc::consts::os::bsd44::AF_UNIX;

use openssl::ssl::{SslStream, SocketIo, SslContext};
use openssl::ssl::error::SslError;
use openssl::bio::SocketBio;

use utils::is_readable::IsReadable;
use utils::socket::ChannelToSocket;

use syscalls;

pub struct SslChannel
{
	stream: Arc<Mutex<SslStream<SocketIo>>>,
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
		
		let ciphertext_fd = ChannelToSocket::new_from(AF_UNIX, SOCK_DGRAM, 0, 
			(ciphertext_tx, ciphertext_rx)).unwrap();

		let (plaintext_tx,  plaintext_rx)  = plaintext_ch;

		debug!("{} SSL pre handshake 1/2 ciphertext_fd={}", is_server, ciphertext_fd);
		let stream = match is_server {
			true => try!(SslStream::new_server_from_socket(&ctx, ciphertext_fd)),
			false => try!(SslStream::new_from_socket(&ctx, ciphertext_fd)),
		};
		info!("{} SSL handshake done! 2/2", is_server);

		syscalls::set_blocking(ciphertext_fd, false).unwrap();
		
		let channel = SslChannel {
			stream: Arc::new(Mutex::new(stream)),
			is_server: is_server,
		};

		channel.spawn_read(plaintext_tx, is_readable);
		channel.spawn_write(plaintext_rx);

		Ok(channel)
	}

	fn spawn_write(&self,
	               plaintext_rx:  Receiver<Vec<u8>>)
	{
		let stream = self.stream.clone();
		let is_server = self.is_server;

		thread::Builder::new().name("SslChannel::spawn_write".to_string()).spawn(move || {
			for buf in plaintext_rx.iter() {
				debug!("{} plaintext_rx {} 1/3", is_server, buf.len());
				let mut s = stream.lock().unwrap();

				debug!("{} plaintext_rx calling SSL_write... 2/3", is_server);
				let len = (*s).write(&buf[..]).unwrap(); // blocking?
				(*s).flush().unwrap();

				debug!("{} plaintext_rx SSL_written len={} 3/3", is_server, len);
				assert_eq!(len, buf.len())
			}
			panic!("fin");
		}).unwrap();
	}

	fn spawn_read(&self,
	              plaintext_tx: Sender<Vec<u8>>,
	              is_readable:  IsReadable)
	{
		let stream = self.stream.clone();
		let is_server = self.is_server;

		let is_readable = is_readable.unpack();
		thread::Builder::new().name("SslChannel::spawn_read".to_string()).spawn(move || {
			loop {
				let mut buf = vec![0; 8*1024];
				debug!("{} SSL_read: wait 1/2", is_server);

				let mut blk = || {
					let mut s = stream.lock().unwrap();
					debug!("ssl rbio pending: {}", (*s.ssl.get_rbio::<SocketBio>()).pending());

					let len = s.read(&mut buf[..]).unwrap();
					debug!("{} SSL_read: done (len={}) 2/2", is_server, len);

					if len > 0 {
						buf.truncate(len);
						plaintext_tx.send(buf.clone()).unwrap();
					}
				};

				let &(ref lock, ref cvar) = &*is_readable;

				let mut readable = lock.lock().unwrap();
				let mut s = stream.lock().unwrap();
				while !*readable && (*s).pending() == 0 {
					drop(s);
					readable = cvar.wait(readable).unwrap();
					s = stream.lock().unwrap();
				}
				drop(s);

				blk();

				*readable = false;
			}
		}).unwrap();
	}
}
