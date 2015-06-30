use openssl::crypto;
use openssl::crypto::hash;
use openssl::crypto::hmac;

use ecdh;
use ecdh::ECDH;

const HMAC_HASH: hash::Type = hash::Type::SHA512;
const CRYPTO:    crypto::symm::Type = crypto::symm::Type::AES_256_CBC;

#[derive(Clone)]
pub struct SharedSecret {
	key:  Vec<u8>,
	iv:   Vec<u8>,
	hash: Vec<u8>,
}

impl SharedSecret {
	/// bloat up 512-bit shared ECDH key to 768 bits (= 3*256 bits = key, IC, hash)
	pub fn new<'a>(local_private_key: &'a ecdh::PrivateKey, remote_public_key: &'a ecdh::PublicKey) -> SharedSecret
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

	pub fn encrypt_then_mac(&self, plaintext: &Vec<u8>) -> Vec<u8> {
		let ciphertext = crypto::symm::encrypt(CRYPTO, &self.key[..], self.iv.to_vec(), plaintext);
		let mac = hmac::hmac(HMAC_HASH, &self.hash[..], &ciphertext[..]);

		mac.into_iter().chain(ciphertext.into_iter()).collect()
	}

	pub fn decrypt(&self, ciphertext: &Vec<u8>)
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
