use std::error;
use std::io::{Cursor,Write};
use std::os::unix::io::RawFd;
use std::cmp::Ordering;
use std::thread::{spawn,sleep_ms};
use std::sync::{Arc,Mutex};
use std::fmt;
use std::fmt::{Formatter,Display};
use std::collections::HashMap;

use rustc_serialize::hex::{ToHex,FromHex};

use libc;
use openssl::ssl::error::SslError;
use openssl::crypto::pkey::{PKey,Parts};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use time;
use time::Duration;
use ecdh;
use dbus::{Message,MessageItem,BusType};
use dbus::Connection as DbusConnection;
use parse_peers::Peer;

use utils::retry::retry;
use utils::convert_dbus_item::ConvertDbusItem;
use connection::{Connection, ControllingMode};
use shared_secret::SharedSecret;

const TIME_LEN: usize = 64/8;

const BULLETIN_BOARD_ID: &'static str = "intercom_v1";

#[derive(Debug)]
pub struct ConnectError {
	pub description: String,
	pub cause:       Option<Box<error::Error>>,
}

impl ConnectError {
	fn new(msg: &str) -> ConnectError {
		ConnectError {
			description: msg.to_string(),
			cause: None,
		}
	}
}

impl Display for ConnectError {
	fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
		write!(fmt, "{:?}", self)
	}
}

impl error::Error for ConnectError {
	fn description(&self) -> &str {
		&self.description[..]
	}

	fn cause(&self) -> Option<&error::Error> {
		self.cause.as_ref()
		          .map(|e| &**e)
	}
}

macro_rules! try_msg {
	($desc:expr, $expr:expr) => (match $expr {
		Result::Ok(val)  => val,
		Result::Err(err) => {
			let error = ConnectError {
				description: format!($desc),
				cause: Some(Box::new(err))
			};
			return Err(error);
		},
	});
	($desc:expr, $expr:expr, $val:expr) => (match $expr {
		Result::Ok(val)  => val,
		Result::Err(err) => {
			let error = ConnectError {
				description: format!($desc),
				cause: $val
			};
			return Err(error);
		},
	});
}

pub struct Intercom {
	peers:             HashMap<String,Peer>,
	local_private_key: ecdh::PrivateKey,
}

fn convert_private_key(key: &ecdh::PrivateKey) -> Result<PKey,SslError> {
	let mut buf = Cursor::new(vec![]);
	key.to_pem(&mut buf).unwrap();
	buf.set_position(0);
	PKey::private_key_from_pem(&mut buf)
}

fn convert_public_key(key: &ecdh::PublicKey) -> PKey {
	let ptr = key.to_evp_pkey().unwrap() as *mut libc::c_void; // TODO: free ptr!!!
	PKey::from_handle(ptr, Parts::Public)
}


impl Intercom {
	pub fn new(local_private_key: &Vec<u8>, peers: HashMap<String,Peer>) -> Result<Intercom,()> {
		let local_private_key = ecdh::PrivateKey::from_vec(&local_private_key).map_err(|_| ())
			.unwrap_or_else(|_| ecdh::PrivateKey::generate().unwrap()); // TODO: just generate a new key?! really!?

		let local_public_key = local_private_key.get_public_key();
		info!("My public key is: {:?}", local_public_key.to_vec().to_hex());

		Ok(Intercom {
			peers:             peers,
			local_private_key: local_private_key,
		})
	}

	pub fn connect_to_peer(&self,
	                       socket_type:       i32,
	                       peername:          String,
	                       local_app_id:      String,
	                       remote_app_id:     String,
	                       timeout:           Duration)
		-> Result<RawFd, ConnectError>
	{
		if let Some(pub_key) = self.peers.get(&peername[..]) {
			self.connect(socket_type,
			             &pub_key.public_key,
			             local_app_id,
			             remote_app_id,
			             timeout)
		} else {
			Err(ConnectError {
				description: format!("Unkown peername {:?}", peername),
				cause: None,
			})
		}
	}

	pub fn connect_to_key(&self,
	                      socket_type:       i32,
	                      remote_public_key: String,
	                      local_app_id:      String,
	                      remote_app_id:     String,
	                      timeout:           Duration)
		-> Result<RawFd, ConnectError>
	{
		debug!("remote_public_key[..].from_hex()={:?}",remote_public_key[..].from_hex());
		let remote_public_key = remote_public_key[..].from_hex().unwrap();
		let remote_public_key = ecdh::PublicKey::from_vec(&remote_public_key).unwrap();

		self.connect(socket_type, &remote_public_key, local_app_id, remote_app_id, timeout)
	}

	fn connect(&self,
	           socket_type:       i32,
	           remote_public_key: &ecdh::PublicKey,
	           local_app_id:      String,
	           remote_app_id:     String,
	           timeout:           Duration)
		-> Result<RawFd, ConnectError>
	{
		let shared_secret = SharedSecret::new(&self.local_private_key,
		                                      &remote_public_key);

		let local_public_key = self.local_private_key.get_public_key();
		let controlling_mode = if local_public_key.to_vec() > remote_public_key.to_vec() {
			ControllingMode::Client
		} else {
			ControllingMode::Server
		};

		let private_key = convert_private_key(&self.local_private_key).unwrap();
		let public_key = convert_public_key(&remote_public_key);

		let mut conn = try!(Connection::new(socket_type,
		                                    private_key,
		                                    public_key,
		                                    controlling_mode));

		let local_dht_key = Self::generate_dht_key(local_app_id.clone(),
		                                           &self.local_private_key,
		                                           &remote_public_key);

		let local_credentials     = conn.get_local_credentials();
		let shared_secret_publish = shared_secret.clone();
		
		let continue_publishing = Arc::new(Mutex::new(true));
		let cont = continue_publishing.clone();

		spawn(move || {
			let continue_publishing = cont;
			let shared_secret = shared_secret_publish;

			Self::publish_credentials(local_dht_key.clone(),
			                          &shared_secret,
			                          local_credentials.clone()).unwrap();
			sleep_ms(15*1000);

			while *continue_publishing.lock().unwrap() {
				debug!("publishing {:?}", local_credentials);
				Self::publish_credentials(local_dht_key.clone(),
				                          &shared_secret,
				                          local_credentials.clone()).unwrap();
				debug!("published");

				sleep_ms(60*1000);
			}
			debug!("stopping publishing");
		});

		let retry_time = Duration::seconds(5);
		let result = retry(timeout, retry_time, || {
			debug!("retrying");
			let remote_credentials = try!(self.get_remote_credentials(&shared_secret,
				remote_app_id.clone(), &remote_public_key));
			info!("server={:?} remote credentials are {:?}", controlling_mode, String::from_utf8(remote_credentials.clone()));

			let fd = try!(conn.establish_connection(remote_credentials));
			Ok(fd)
		});

		*continue_publishing.lock().unwrap() = false;
		result
	}

	fn get_remote_credentials(&self,
	                          shared_secret:     &SharedSecret, 
	                          app_id:            String,
	                          remote_public_key: &ecdh::PublicKey)
		-> Result<Vec<u8>,ConnectError>
	{
		let local_public_key = self.local_private_key.get_public_key();

		let key:Vec<u8> = remote_public_key.to_vec().into_iter()
			.chain(local_public_key.to_vec().into_iter())
			.chain(app_id.into_bytes().into_iter())
			.collect();

		let session = BusType::Session;
		let conn    = DbusConnection::get_private(session).unwrap();
		let app_id  = MessageItem::Str(BULLETIN_BOARD_ID.to_string());
		let mut msg = Message::new_method_call("org.manuel.BulletinBoard",
		                                       "/",
		                                       "org.manuel.BulletinBoard",
		                                       "Get").unwrap();
		msg.append_items(&[app_id, key.to_dbus_item()]);
		let reply = try_msg!("",
		                         conn.send_with_reply_and_block(msg, 60000));

		match reply.get_items().get(0) {
			Some(&MessageItem::Array(ref items, ref t)) if t == "ay" => {
				let decrypt = |vec| shared_secret.decrypt(&vec);

				info!("Found {} potential remote credentials.", items.len());
				let mut values:Vec<Vec<u8>> = items.iter()
					.map(|item| Vec::<u8>::from_dbus_item(item).and_then(&decrypt))
					.filter_map(|o| o)
					.filter(|vec| vec.len() > TIME_LEN)
					.collect();

				let now = time::now_utc().to_timespec();
				let read_age = |v:&Vec<u8>| {
					let mut c = Cursor::new(v.clone());
					c.set_position(0);
					let timestamp = c.read_i64::<LittleEndian>().ok();
					timestamp.map(|ts| now.sec - ts)
				};

				values.sort_by(|x,y| {
					let a = read_age(x);
					let b = read_age(y);

					match (a,b) {
						(Some(a), Some(b)) => a.cmp(&b),
						(None, None)    => Ordering::Equal,
						(None, Some(_)) => Ordering::Less,
						(Some(_), None) => Ordering::Greater,
					}
				});

				match values.pop() {
					None => Err(ConnectError::new("Remote credentials not found.")),
					Some(latest) => {
						let (timestamp, credentials) = latest.split_at(TIME_LEN);

						match read_age(&timestamp.to_vec()) {
							None => Err(ConnectError::new("Remote credentials not found.")),
							Some(age_sec) => {
								info!("Latest remote credentials are {}sec old", age_sec);

								Ok(credentials.to_vec())
							}
						}

					}
				}
			},
			_ => {
				Err(ConnectError::new("org.manuel.BulletinBoard.Get() failed!"))
			},
		}
	}

	fn generate_dht_key(app_id:            String,
	                    local_private_key: &ecdh::PrivateKey,
	                    remote_public_key: &ecdh::PublicKey)
		-> Vec<u8>
	{
		let local_public_key = local_private_key.get_public_key();
		let key:Vec<u8> = local_public_key.to_vec().into_iter()
			.chain(remote_public_key.to_vec().into_iter())
			.chain(app_id.into_bytes().into_iter())
			.collect();

		key
	}

	fn publish_credentials(dht_key:           Vec<u8>,
	                       shared_secret:     &SharedSecret,
	                       local_credentials: String)
		-> Result<(Vec<u8>),ConnectError>
	{
		let now = time::now_utc().to_timespec();

		let mut plaintext_value = Cursor::new(vec![]);
		plaintext_value.write_i64::<LittleEndian>(now.sec).unwrap();
		plaintext_value.write(&local_credentials.into_bytes()[..]).unwrap();

		let ciphertext_value = shared_secret.encrypt_then_mac(plaintext_value.get_ref());

		let conn = DbusConnection::get_private(BusType::Session).unwrap();

		let mut msg = Message::new_method_call("org.manuel.BulletinBoard", "/",
			"org.manuel.BulletinBoard", "Put").unwrap();
		let app_id = MessageItem::Str(BULLETIN_BOARD_ID.to_string());
		msg.append_items(&[app_id,
		                   dht_key.to_dbus_item(),
		                   ciphertext_value.to_dbus_item()]);
		try_msg!("org.manuel.BulletinBoard.Put() failed!",
		         conn.send_with_reply_and_block(msg, 60000));

		Ok(ciphertext_value)
	}
}
