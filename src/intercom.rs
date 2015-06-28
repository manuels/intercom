use std::io::{Cursor,Write};
use std::os::unix::io::RawFd;
use std::cmp::Ordering;
use std::thread::{spawn,sleep_ms};

use rustc_serialize::hex::{ToHex,FromHex};

use libc;
use openssl::crypto::hash;
use openssl::crypto;
use openssl::crypto::hmac;
use openssl::ssl::error::SslError;
use openssl::crypto::pkey::{PKey,Parts};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use time;
use time::Duration;
use ecdh;
use ecdh::ECDH;
use dbus::{Message,MessageItem,BusType};
use dbus::Connection as DbusConnection;

use utils::convert_dbus_item::ConvertDbusItem;
use connection::Connection;
use utils::retry::retry;
use utils::ignore;

const TIME_LEN: usize = 64/8;

const HMAC_HASH: hash::Type = hash::Type::SHA512;
const CRYPTO:    crypto::symm::Type = crypto::symm::Type::AES_256_CBC;

const BULLETIN_BOARD_ID: &'static str = "intercom_v1";

#[derive(Debug)]
pub enum ConnectError {
	RemoteCredentialsNotFound,
	IceConnectFailed,
	SslError(SslError),
	DHTError,
	Internal(&'static str),
	FOO,
}

pub struct Intercom {
	local_private_key: ecdh::PrivateKey,
}

#[derive(Clone)]
struct SharedSecret {
	key:  Vec<u8>,
	iv:   Vec<u8>,
	hash: Vec<u8>,
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
	pub fn new(local_private_key: &Vec<u8>) -> Result<Intercom,()> {
		let local_private_key = String::from_utf8(local_private_key.clone()).unwrap();
		let local_private_key = (&local_private_key[..]).from_hex().map_err(|_| ())
			.and_then(|v| ecdh::PrivateKey::from_vec(&v).map_err(|_| ()))
			.unwrap_or_else(|_| ecdh::PrivateKey::generate().unwrap()); // TODO: just generate a new key?! really!?

		let local_public_key = local_private_key.get_public_key();
		debug!("My private key is: {:?}", local_private_key.to_vec().to_hex());
		debug!("My public key is: {:?}", local_public_key.to_vec().to_hex());

		Ok(Intercom {
			local_private_key: local_private_key
		})
		
	}

	pub fn connect(&self, socket_type: i32, remote_public_key: String, 
	               app_id: String,  timeout: Duration)
		-> Result<RawFd, ConnectError>
	{
		debug!("remote_public_key[..].from_hex()={:?}",remote_public_key[..].from_hex());
		let remote_public_key = remote_public_key[..].from_hex().unwrap();
		let remote_public_key = ecdh::PublicKey::from_vec(&remote_public_key).unwrap();

		let shared_secret = SharedSecret::new(&self.local_private_key, &remote_public_key);

		let local_public_key = self.local_private_key.get_public_key();
		let controlling_mode = local_public_key.to_vec() > remote_public_key.to_vec();

		let private_key = convert_private_key(&self.local_private_key).unwrap();
		let public_key = convert_public_key(&remote_public_key);
		let mut conn = try!(Connection::new(socket_type, private_key, public_key,
			                           controlling_mode));

		let dht_key = Self::generate_dht_key(app_id.clone(),
		                            &self.local_private_key, &remote_public_key);

		let local_credentials = conn.get_local_credentials();
		let shared_secret_publish = shared_secret.clone();
		
		let publisher = spawn(move || {
			let shared_secret = shared_secret_publish;

			Self::publish_credentials(dht_key.clone(), &shared_secret,
				local_credentials.clone());
			sleep_ms(15*1000);

			loop {
				debug!("publishing {:?}", String::from_utf8(local_credentials.clone()));
				Self::publish_credentials(dht_key.clone(), &shared_secret,
					local_credentials.clone());
				debug!("published");

				sleep_ms(60*1000);
			}
		});

		let retry_time = Duration::seconds(5);
		let result = retry(timeout, retry_time, || {
			debug!("retrying");
			let remote_credentials = try!(self.get_remote_credentials(&shared_secret,
				app_id.clone(), &remote_public_key));
			info!("server={:?} remote credentials are {:?}", controlling_mode, String::from_utf8(remote_credentials.clone()));

			let fd = try!(conn.establish_connection(remote_credentials));
			Ok(fd)
		});

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

		let conn = DbusConnection::get_private(BusType::Session).unwrap();
		let mut msg = Message::new_method_call("org.manuel.BulletinBoard", "/",
			"org.manuel.BulletinBoard", "Get").unwrap();
		let app_id = MessageItem::Str(BULLETIN_BOARD_ID.to_string());
		msg.append_items(&[app_id, key.to_dbus_item()]);
		let mut reply = try!(conn.send_with_reply_and_block(msg, 60000)
			.map_err(|e| {info!("{:?}", e); ConnectError::DHTError}));

		match reply.get_items().get(0) {
			Some(&MessageItem::Array(ref items, ref t)) if t == "ay" => {
				let decrypt = |vec| shared_secret.decrypt(&vec);

				info!("Found {} potential remote credentials.", items.len());
				let mut values:Vec<Vec<u8>> = items.iter()
					.map(|item| Vec::<u8>::from_dbus_item(item).and_then(&decrypt))
					.filter(Option::is_some)
					.map(Option::unwrap)
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
					None => Err(ConnectError::RemoteCredentialsNotFound),
					Some(latest) => {
						let (timestamp, credentials) = latest.split_at(TIME_LEN);

						match read_age(&timestamp.to_vec()) {
							None => Err(ConnectError::RemoteCredentialsNotFound),
							Some(age_sec) => {
								info!("Lastest remote credentials are {}sec old", age_sec);

								Ok(credentials.to_vec())
							}
						}

					}
				}
			},
			_ => {
				warn!("org.manuel.BulletinBoard.Get() failed!");
				Err(ConnectError::FOO)
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
	                       local_credentials: Vec<u8>)
		-> Result<(Vec<u8>),ConnectError>
	{
		let mut plaintext_value = Cursor::new(vec![]);
		let now = time::now_utc().to_timespec();
		plaintext_value.write_i64::<LittleEndian>(now.sec).unwrap();
		plaintext_value.write(&local_credentials[..]).unwrap();

		let ciphertext_value = shared_secret.encrypt_then_mac(plaintext_value.get_ref());

		let conn = DbusConnection::get_private(BusType::Session).unwrap();

		let mut msg = Message::new_method_call("org.manuel.BulletinBoard", "/",
			"org.manuel.BulletinBoard", "Put").unwrap();
		let app_id = MessageItem::Str(BULLETIN_BOARD_ID.to_string());
		msg.append_items(&[app_id,
		                   dht_key.to_dbus_item(),
		                   ciphertext_value.to_dbus_item()]);
		try!(conn.send_with_reply_and_block(msg, 60000)
			.map_err(|e| {warn!("{:?}", e); ConnectError::DHTError}));

		Ok(ciphertext_value)
	}
}

impl SharedSecret {
	/// bloat up 512-bit shared ECDH key to 768 bits (= 3*256 bits = key, IC, hash)
	fn new<'a>(local_private_key: &'a ecdh::PrivateKey, remote_public_key: &'a ecdh::PublicKey) -> SharedSecret
	{
		let shared = ECDH::compute_key(local_private_key, remote_public_key).unwrap();

		assert_eq!(shared.len(), 512/8);
		let (key, seed) = shared[..].split_at(256/8);

		assert_eq!(seed.len(), 256/8);
		let typ = hash::Type::SHA512;
		let md  = hash::hash(typ, seed);
		let (iv, hash) = md.split_at(256/8);

		assert_eq!(key.len(),  256/8);
		assert_eq!(iv.len(),   256/8);
		assert_eq!(hash.len(), 256/8);

		SharedSecret {
			key:  key[..256/8].to_vec(),
			iv:   iv[..256/8].to_vec(),
			hash: hash[..256/8].to_vec(),
		}
	}

	fn encrypt_then_mac(&self, plaintext: &Vec<u8>) -> Vec<u8> {
		let ciphertext = crypto::symm::encrypt(CRYPTO, &self.key[..], self.iv.to_vec(), plaintext);
		let mac = hmac::hmac(HMAC_HASH, &self.hash[..], &ciphertext[..]);

		mac.into_iter().chain(ciphertext.into_iter()).collect()
	}

	fn decrypt(&self, ciphertext: &Vec<u8>)
		-> Option<Vec<u8>>
	{
		if ciphertext.len() < HMAC_HASH.md_len() {
			debug!("Credentials are invalid (too short)");
			return None;
		}

		let (actual_hmac, ctxt) = ciphertext.split_at(HMAC_HASH.md_len());
		let expected_hmac = hmac::hmac(HMAC_HASH, &self.hash[..], &ctxt[..]);

		let plaintext = crypto::symm::decrypt(CRYPTO, &self.key[..], self.iv.to_vec(), &ctxt[..]);

		assert_eq!(actual_hmac.len(), expected_hmac.len());
		if crypto::memcmp::eq(&actual_hmac, &expected_hmac) {
			debug!("Credentials are valid");
			Some(plaintext)
		} else {
			debug!("Credentials are invalid (incorrect hmac)");
			debug!("actual hmac  ={:?}", actual_hmac);
			debug!("expected hmac={:?}", expected_hmac);
			
			None
		}
	}
}

#[test]
fn test_shared_secret() {
	let public_key  = vec![48u8, 51, 48, 49, 48, 66, 67, 54, 53, 56, 51, 52, 65, 56, 54, 50, 65, 65, 57, 65, 51, 51, 69, 51, 65, 51, 48, 69, 52, 70, 57, 50, 49, 51, 57, 67, 50, 56, 49, 70, 68, 49, 48, 52, 54, 49, 54, 50, 51, 56, 66, 70, 67, 48, 49, 54, 68, 65, 66, 53, 69, 49, 48, 57, 68, 54, 69, 70, 48, 55, 55, 50, 55, 70, 69, 69, 51, 50, 48, 70, 69, 67, 54, 65, 53, 52, 69, 57, 49, 66, 53, 67, 49, 52, 54, 52, 49, 53, 54, 51, 48, 50, 50, 65, 57, 69, 50, 51, 53, 53, 66, 48, 65, 70, 65, 49, 54, 50, 54, 52, 66, 51, 68, 70, 65, 69, 50, 49, 55, 55, 66, 55, 70, 53];
	let private_key = vec![54u8, 69, 51, 50, 69, 54, 48, 50, 54, 69, 56, 66, 54, 69, 52, 48, 54, 53, 51, 57, 57, 54, 65, 69, 70, 70, 57, 65, 69, 49, 68, 55, 53, 49, 51, 66, 69, 52, 55, 55, 56, 56, 65, 68, 53, 67, 49, 51, 51, 51, 53, 65, 48, 52, 67, 54, 54, 65, 51, 57, 57, 68, 53, 51, 65, 53, 70, 65, 50, 55, 50, 66, 54, 55, 55, 68, 66, 54, 55, 48, 69, 66, 65, 50, 66, 66, 52, 70, 49, 67, 56, 49, 57, 52, 49, 57, 68, 50, 55, 67, 55, 66, 67, 53, 68, 51, 52, 56, 51, 56, 49, 49, 54, 56, 55, 49, 68, 56, 49, 48, 55, 50, 56, 66, 50, 49, 65, 55, 67, 66];

	let public_key = ecdh::PublicKey::from_vec(&public_key).unwrap();
	let private_key = ecdh::PrivateKey::from_vec(&private_key).unwrap();

	let shared_secret = SharedSecret::new(&private_key, &public_key);

	let plaintext = "foobar".bytes().collect();
	let ciphertext = shared_secret.encrypt_then_mac(&plaintext);

	assert_eq!(Some(plaintext), shared_secret.decrypt(&ciphertext));

	let manipulated_ciphertext = ciphertext.into_iter().chain(vec![1].into_iter()).collect();
	assert_eq!(None, shared_secret.decrypt(&manipulated_ciphertext));
}

#[test]
fn test_shared_secret_manipulated() {
	let public_key  = vec![48u8, 51, 48, 49, 48, 66, 67, 54, 53, 56, 51, 52, 65, 56, 54, 50, 65, 65, 57, 65, 51, 51, 69, 51, 65, 51, 48, 69, 52, 70, 57, 50, 49, 51, 57, 67, 50, 56, 49, 70, 68, 49, 48, 52, 54, 49, 54, 50, 51, 56, 66, 70, 67, 48, 49, 54, 68, 65, 66, 53, 69, 49, 48, 57, 68, 54, 69, 70, 48, 55, 55, 50, 55, 70, 69, 69, 51, 50, 48, 70, 69, 67, 54, 65, 53, 52, 69, 57, 49, 66, 53, 67, 49, 52, 54, 52, 49, 53, 54, 51, 48, 50, 50, 65, 57, 69, 50, 51, 53, 53, 66, 48, 65, 70, 65, 49, 54, 50, 54, 52, 66, 51, 68, 70, 65, 69, 50, 49, 55, 55, 66, 55, 70, 53];
	let private_key = vec![54u8, 69, 51, 50, 69, 54, 48, 50, 54, 69, 56, 66, 54, 69, 52, 48, 54, 53, 51, 57, 57, 54, 65, 69, 70, 70, 57, 65, 69, 49, 68, 55, 53, 49, 51, 66, 69, 52, 55, 55, 56, 56, 65, 68, 53, 67, 49, 51, 51, 51, 53, 65, 48, 52, 67, 54, 54, 65, 51, 57, 57, 68, 53, 51, 65, 53, 70, 65, 50, 55, 50, 66, 54, 55, 55, 68, 66, 54, 55, 48, 69, 66, 65, 50, 66, 66, 52, 70, 49, 67, 56, 49, 57, 52, 49, 57, 68, 50, 55, 67, 55, 66, 67, 53, 68, 51, 52, 56, 51, 56, 49, 49, 54, 56, 55, 49, 68, 56, 49, 48, 55, 50, 56, 66, 50, 49, 65, 55, 67, 66];

	let public_key = ecdh::PublicKey::from_vec(&public_key).unwrap();
	let private_key = ecdh::PrivateKey::from_vec(&private_key).unwrap();

	let shared_secret = SharedSecret::new(&private_key, &public_key);

	let plaintext = "foobar".bytes().collect();
	let ciphertext = shared_secret.encrypt_then_mac(&plaintext);

	let manipulated_ciphertext = ciphertext.into_iter().chain(vec![1].into_iter()).collect();
	assert_eq!(None, shared_secret.decrypt(&manipulated_ciphertext));
}
