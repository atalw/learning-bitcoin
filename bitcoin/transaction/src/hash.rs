
use ripemd::Ripemd160;
use sha2::{Sha256, Digest};

pub fn create_hash160(bytes: &[u8]) -> Vec<u8> {
	let sha256 = Sha256::digest(bytes);
	let hash160 = Ripemd160::digest(sha256);
	return hash160.to_vec()
}

pub fn create_hash256(bytes: &[u8]) -> Vec<u8> {
	let sha256 = Sha256::digest(bytes);
	let hash256 = Sha256::digest(sha256);
	return hash256.to_vec()
}
