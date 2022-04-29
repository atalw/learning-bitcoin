
use ripemd::Ripemd160;
use sha2::{Sha256, Digest};

// Note: hash of hex_string != hash of bytes. We need to hash at byte level.
pub fn hash160(bytes: &[u8]) -> Vec<u8> {
	let sha256 = Sha256::digest(bytes);
	let hash160 = Ripemd160::digest(sha256);
	return hash160.to_vec()
}

pub fn hash256(bytes: &[u8]) -> Vec<u8> {
	let sha256 = Sha256::digest(bytes);
	let hash256 = Sha256::digest(sha256);
	return hash256.to_vec()
}
