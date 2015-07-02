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
	let public_key  = vec![2, 1, 0, 163, 215, 7, 212, 111, 65, 12, 71, 241, 53, 52, 251, 41, 237, 3, 29, 101, 63, 116, 130, 150, 64, 159, 132, 150, 85, 202, 191, 31, 227, 17, 30, 34, 46, 102, 166, 187, 133, 4, 84, 239, 190, 162, 174, 161, 40, 3, 203, 213, 79, 238, 16, 123, 90, 254, 108, 134, 181, 104, 112, 100, 116, 20, 238];
	let private_key = vec![1, 220, 254, 121, 176, 90, 169, 167, 226, 22, 16, 143, 36, 56, 183, 61, 167, 195, 174, 191, 140, 134, 86, 16, 123, 213, 40, 103, 174, 46, 250, 54, 119, 172, 247, 135, 144, 60, 99, 14, 242, 129, 212, 64, 121, 172, 200, 4, 121, 60, 129, 126, 58, 16, 23, 225, 56, 245, 56, 32, 109, 226, 94, 27, 162, 83];

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
	let public_key  = vec![2, 1, 0, 163, 215, 7, 212, 111, 65, 12, 71, 241, 53, 52, 251, 41, 237, 3, 29, 101, 63, 116, 130, 150, 64, 159, 132, 150, 85, 202, 191, 31, 227, 17, 30, 34, 46, 102, 166, 187, 133, 4, 84, 239, 190, 162, 174, 161, 40, 3, 203, 213, 79, 238, 16, 123, 90, 254, 108, 134, 181, 104, 112, 100, 116, 20, 238];
	let private_key = vec![1, 220, 254, 121, 176, 90, 169, 167, 226, 22, 16, 143, 36, 56, 183, 61, 167, 195, 174, 191, 140, 134, 86, 16, 123, 213, 40, 103, 174, 46, 250, 54, 119, 172, 247, 135, 144, 60, 99, 14, 242, 129, 212, 64, 121, 172, 200, 4, 121, 60, 129, 126, 58, 16, 23, 225, 56, 245, 56, 32, 109, 226, 94, 27, 162, 83];

	let public_key = ecdh::PublicKey::from_vec(&public_key).unwrap();
	let private_key = ecdh::PrivateKey::from_vec(&private_key).unwrap();

	let shared_secret = SharedSecret::new(&private_key, &public_key);

	let plaintext = "foobar".bytes().collect();
	let ciphertext = shared_secret.encrypt_then_mac(&plaintext);

	let manipulated_ciphertext = ciphertext.into_iter().chain(vec![1].into_iter()).collect();
	assert_eq!(None, shared_secret.decrypt(&manipulated_ciphertext));
}
