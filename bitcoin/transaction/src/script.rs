/// Code ideas from https://github.com/rust-bitcoin/rust-bitcoin/blob/master/src/blockdata/script.rs

use std::error::Error;
use std::io::{BufRead, Cursor};
use crate::txio::{Encodable, Decodable, HexBytes, ReadExt, UserReadExt};
use crate::{Serialize, opcodes, Deserialize, hash};
use std::fmt;

// TODO: Q. How do I organize the code so that I can have a ScriptPubKey and ScriptSig type with
// different fields but they both are Scripts? I want them to inherit the Serialize and Deserialize
// impl for Script so that I don't have to reimplement it. Not sure how to type limit a generic
// impl to achieve this.
// Ans 1. I've created a Script trait and implementing it on ScriptPubKey and ScriptSig using a
// macro. It's effectively writing the code twice though. Is there a better way to do this?

/// Script is the programming language used in Bitcoin to construct a ScriptSig and ScriptPubKey.
/// While the semantics of the language haven't been handled yet, parsing to and from has been
/// implemented.
pub trait Script {
	/// Convert hex-formatted bytes into a Script type.
	fn from_bytes(bytes: HexBytes) -> Self where Self: Sized;
	/// Convert a hex string into a Script type.
	fn from_str(hex: &str) -> Self where Self: Sized;

	/// Create a new Pay-to-Script-Hash (P2SH) type script given an original script. This function does a hash160 of
	/// the provided raw script before generating the P2SH script.
	fn new_p2sh(original_script: HexBytes) -> Self where Self: Sized;
	/// Create a new Pay-to-Public-Key-Hash (P2PKH) type script given a public key.
	fn new_p2pkh(public_key: HexBytes) -> Self where Self: Sized;

	/// Checks whether a script pubkey is a P2SH output.
	fn is_p2sh(&self) -> bool;
	/// Checks whether a script pubkey is a P2PKH output.
	fn is_p2pkh(&self) -> bool;

	/// Generate an Base58 encoded address of a given script. 
	/// Note: at the moment, this only generates an address for P2SH and P2PKH script types.
	fn get_address(&self) -> Option<String>;
	/// Determine the script type.
	fn get_type(&self) -> ScriptType;

	/// Creates an assembly-formatted string of the input Script. Right now this is only used for
	/// display purposes. 
	fn as_asm(&self) -> String;
}

macro_rules! impl_script_for {
	($ty: ty) => {
		impl Script for $ty {
			fn from_bytes(bytes: HexBytes) -> Self {
				<$ty>::new(bytes)
			}

			fn from_str(hex: &str) -> Self {
				let bytes = match hex.decode_hex_be() {
					Ok(b) => b,
					Err(e) => panic!("{}", e)
				};
				<$ty>::from_bytes(bytes)
			}

			fn new_p2sh(original_script: HexBytes) -> Self {
				let script_hash = hash::hash160(&original_script);
				let mut script_builder = ScriptBuilder::new();
				script_builder.push_opcode(opcodes::all::OP_HASH160);
				script_builder.push_script_hash(&script_hash);
				script_builder.push_opcode(opcodes::all::OP_EQUAL);
				script_builder.into_script()
			}

			fn new_p2pkh(public_key: HexBytes) -> Self {
				let public_key_hash = hash::hash160(&public_key);
				let mut script_builder = ScriptBuilder::new();
				script_builder.push_opcode(opcodes::all::OP_DUP);
				script_builder.push_opcode(opcodes::all::OP_HASH160);
				script_builder.push_script_hash(&public_key_hash);
				script_builder.push_opcode(opcodes::all::OP_EQUALVERIFY);
				script_builder.push_opcode(opcodes::all::OP_CHECKSIG);
				script_builder.into_script()
			}

			#[inline]
			fn is_p2sh(&self) -> bool {
				self.script.len() == 23
					&& self.script[0] == opcodes::all::OP_HASH160.into_u8()
					&& self.script[1] == opcodes::all::OP_PUSHBYTES_20.into_u8()
					&& self.script[22] == opcodes::all::OP_EQUAL.into_u8()
			}

			#[inline]
			fn is_p2pkh(&self) -> bool {
				self.script.len() == 25
					&& self.script[0] == opcodes::all::OP_DUP.into_u8()
					&& self.script[1] == opcodes::all::OP_HASH160.into_u8()
					&& self.script[2] == opcodes::all::OP_PUSHBYTES_20.into_u8()
					&& self.script[23] == opcodes::all::OP_EQUALVERIFY.into_u8()
					&& self.script[24] == opcodes::all::OP_CHECKSIG.into_u8()
			}

			fn get_address(&self) -> Option<String> {
				if self.is_p2pkh() {
					let pubkey_hash = &self.script[3..23];
					let mut bytes = vec![111]; // using testnet prefix
					bytes.extend_from_slice(pubkey_hash);
					let checksum = &hash::hash256(&bytes.clone())[..4];
					bytes.extend_from_slice(&checksum);
					assert_eq!(bytes.len(), 25);
					Some(bs58::encode(bytes).into_string())
				} else if self.is_p2sh() {
					let pubkey_hash = &self.script[2..22];
					let mut bytes = vec![196]; // using testnet prefix
					bytes.extend_from_slice(pubkey_hash);
					let checksum = &hash::hash256(&bytes.clone())[..4];
					bytes.extend_from_slice(&checksum);
					assert_eq!(bytes.len(), 25);
					Some(bs58::encode(bytes).into_string())
				} else {
					None
				}
			}

			fn get_type(&self) -> ScriptType {
				if self.is_p2sh() {
					ScriptType::P2SH
				} else if self.is_p2pkh() {
					ScriptType::P2PKH
				} else {
					ScriptType::Custom
				}
			}

			fn as_asm(&self) -> String {
				let hexbytes = &self.script;
				let len = hexbytes.len();
				let mut stream = Cursor::new(hexbytes);

				let mut parsed = "".to_string();

				while (stream.position() as usize) < len {
					let b = stream.read_u8_le().expect("won't fail");
					let opcode = opcodes::All::from(b);

					// not sure if this is the correct condition
					if opcode.code == opcodes::all::OP_PUSHBYTES_1.into_u8() {
						let size = stream.read_u8_le().expect("shouldn't fail i think");
						parsed.push_str(&format!("{}", size));
						parsed.push_str(" ");
					} else if opcode.code > opcodes::all::OP_PUSHBYTES_1.into_u8() && opcode.code <= opcodes::all::OP_PUSHBYTES_75.into_u8() {
						let len = opcode.code;
						let script = stream.read_hex_var(len as u64).expect("shouldn't fail i think");
						parsed.push_str(&(*script).encode_hex_be());
						parsed.push_str(" ");
					} else if opcode.code >= opcodes::all::OP_PUSHNUM_1.into_u8() && 
						opcode.code <= opcodes::all::OP_PUSHNUM_15.into_u8() {
							let hex_num = 1 + opcode.code - opcodes::all::OP_PUSHNUM_1.code;
							let dec_num = u32::from_str_radix(&hex_num.to_string(), 16).unwrap();
							parsed.push_str(&format!("{} ", dec_num));
						} else {
							parsed.push_str(&format!("{:02x?} ", opcode));
						}
				}
				parsed.trim_end().to_string()
			}
		}
	};
}

impl_script_for!(ScriptSig);
impl_script_for!(ScriptPubKey);

/// Script Signatures are the unlocking script provided with an input that satisfies the conditions
/// placed by the ScriptPubKey.
pub struct ScriptSig {
	pub script: HexBytes
}

/// Script public keys are the locking script put on an output which prevents others from spending
/// it.
pub struct ScriptPubKey {
	pub script: HexBytes,
	address: Option<String>,
	script_type: Option<ScriptType>
}

pub enum ScriptType {
	P2SH,
	P2PKH,
	Custom
}

pub struct ScriptAsm(Vec<ScriptAsmTokens>);

pub enum ScriptAsmTokens {
	Opcode,
	Hex,
}

impl ScriptSig {
	pub fn new(bytes: HexBytes) -> Self {
		ScriptSig {
			script: bytes,
		}
	}
}

impl ScriptPubKey {
	// FIXME: This is a hack because of a poor architectural choice. I have to instantiate a script 
	// with None address and script_type first. What is the best way to fix this?
	pub fn new(bytes: HexBytes) -> Self {
		let mut script = ScriptPubKey { 
			script: bytes,
			address: None,
			script_type: None
		};

		script.address = script.get_address();
		script.script_type = Some(script.get_type());
		script
	}
}

macro_rules!  impl_serialize_for {
	($ty: ty) => {
		impl Serialize for $ty {
			fn encode_raw<R: BufRead>(mut reader: R) -> Self {
				println!("What type of script do you want to create?");
				println!("1. P2SH");
				println!("2. P2PKH");
				println!("3. Leave empty (00)");
				println!("4. Custom (be careful)");
				println!("Enter option:");
				let option = reader.user_read_u32();

				if option == 1 { // p2sh script
					println!("---- What is the format?");
					println!("---- 1. Script hex");
					println!("---- 2. Script asm");
					let option = reader.user_read_u32();
					let script;
					if option == 1 { // hex
						println!("Enter the unhashed script hex:");
						script = <$ty>::from_bytes(reader.user_read_hex_var());
					} else if option == 2 { // asm
						println!("---- Enter the script in assembly. I'll hash it for you:");
						script = <$ty>::from_bytes(reader.user_read_asm());
					} else { unimplemented!() }
					println!("Original script: {}", script.as_hex());
					<$ty>::new_p2sh(script.as_bytes())
				} else if option == 2 { // p2pkh script
					println!("Enter the public key:");
					let key = <$ty>::from_bytes(reader.user_read_hex_var());
					Self::new_p2pkh(key.as_bytes())
				} else if option == 3 { // empty, useful for signrawtransactionwithwallet
					<$ty>::from_str("00")
				}  else if option == 4 { // custom script
					<$ty>::from_bytes(reader.user_read_hex_var())
				} else {
					todo!()
				}
			}

			fn as_hex(&self) -> String {
				self.script.encode_hex_be()
			}

			fn as_bytes(&self) -> HexBytes {
				self.script.clone()
			}
		}
	};
}

impl_serialize_for!(ScriptSig);
impl_serialize_for!(ScriptPubKey);


macro_rules! impl_deserialize_for {
	($ty: ty) => {
		impl Deserialize for $ty {
			fn decode_raw(bytes: HexBytes) -> Result<Self, Box<dyn Error>> {
				println!("{:02x?}", bytes);
				let len = bytes.len();
				let mut stream = Cursor::new(bytes);

				let mut script_builder = ScriptBuilder::new();

				while (stream.position() as usize) < len {
					let b = stream.read_u8_le().expect("shouldn't fail i think");
					let opcode = opcodes::All::from(b);

					// not sure if this is the correct condition
					if opcode.code == opcodes::all::OP_PUSHBYTES_1.into_u8() {
						let size = stream.read_u8_le().expect("shouldn't fail i think");
						script_builder.push_size(size);
					} else if opcode.code > opcodes::all::OP_PUSHBYTES_1.into_u8() && opcode.code <= opcodes::all::OP_PUSHBYTES_75.into_u8() {
						let len = opcode.code;
						let script = stream.read_hex_var(len as u64).expect("shouldn't fail i think");
						script_builder.push_script_hash(&script)
					} else if opcode.code >= opcodes::all::OP_PUSHNUM_1.into_u8() && 
						opcode.code <= opcodes::all::OP_PUSHNUM_15.into_u8() {
							script_builder.push_opcode(opcode);
						} else {
							script_builder.push_opcode(opcode);
						}
				}
				Ok(script_builder.into_script())
			}
		}
	};
}


impl_deserialize_for!(ScriptSig);
impl_deserialize_for!(ScriptPubKey);


impl PartialEq for ScriptSig {
    fn eq(&self, other: &ScriptSig) -> bool {
		self.script == other.script
	}
}

impl PartialEq for ScriptPubKey {
    fn eq(&self, other: &ScriptPubKey) -> bool {
		self.script == other.script
	}
}


impl fmt::Debug for ScriptSig {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_str("{\n")?;
		f.write_str("\tasm: ")?;
		write!(f, "\"{}\"", self.as_asm())?;
		f.write_str("\n")?;
		f.write_str("\thex: ")?;
		write!(f, "\"{}\"", self.as_hex())?;
		f.write_str("\n}")
	}
}

impl fmt::Debug for ScriptPubKey {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_str("{\n")?;
		f.write_str("\tasm: ")?;
		write!(f, "\"{}\"", self.as_asm())?;
		f.write_str("\n")?;
		f.write_str("\thex: ")?;
		write!(f, "\"{}\"", self.as_hex())?;
		match &self.address {
			Some(s) => {
				f.write_str("\n")?;
				f.write_str("\taddress: ")?;
				write!(f, "\"{}\"", s)?;
			},
			None => {}
		}
		f.write_str("\n")?;
		f.write_str("\ttype: ")?;
		match &self.script_type {
			Some(t) => {
				f.write_str("\n")?;
				f.write_str("\taddress: ")?;
				write!(f, "\"{}\"", t)?;
			},
			None => {}
		}
		f.write_str("\n}")
	}
}

/// Build the script piece by piece
pub struct ScriptBuilder(Vec<u8>);

impl ScriptBuilder {
	pub fn new() -> Self {
		ScriptBuilder(vec![])	
	}

	pub fn into_script<T: Script>(&self) -> T {
		let hexbytes = self.0.clone().into_boxed_slice();
		T::from_bytes(hexbytes)
	}

	pub fn push(&mut self, token: &str) -> Result<(), Box<dyn Error>> {
		let code = opcodes::All::from(token);

		// TODO: Not the best idea but it works for now
		if code == opcodes::all::OP_INVALIDOPCODE {
			let hash = token.decode_hex_be()?;
			self.push_script_hash(&hash);
		} else {
			self.push_opcode(code);
		}

		Ok(())
	}

	fn push_opcode(&mut self, opcode: opcodes::All) {
		self.0.push(opcode.into_u8());
	}

	fn push_script_hash(&mut self, script_hash: &[u8]) {
		self.push_var_int(script_hash.len() as u64);
		self.0.extend(script_hash.iter().cloned());
	}

	// Is there a better way to do this? feels hacky
	fn push_size(&mut self, size: u8) {
		self.push_opcode(opcodes::all::OP_PUSHBYTES_1);
		self.0.push(size);
	}

	/**
	 * Compact Size
	 * https://en.bitcoin.it/wiki/Protocol_documentation#Variable_length_integer
	 * size <  253        -- 1 byte
	 * size <= USHRT_MAX  -- 3 bytes  (253 + 2 bytes)
	 * size <= UINT_MAX   -- 5 bytes  (254 + 4 bytes)
	 * size >  UINT_MAX   -- 9 bytes  (255 + 8 bytes)
	 * fc -> 0-252
	 * fd -> 0000 (253 + 2 bytes)
	 * fe -> 0000 0000 (254 + 4 bytes)
	 * ff -> 0000 0000 0000 0000 (255 + 8 bytes)
	 * check bitcoin/src/serialize.h file
	 */
	// This code is repeated in txio
	fn push_var_int(&mut self, n: u64) {
		if n < opcodes::all::OP_PUSHDATA1.into_u8() as u64 {
			self.0.push(n as u8);
		} else if n < 0x100 { // 256
			self.0.push(opcodes::all::OP_PUSHDATA1.into_u8());
			self.0.push(n as u8);
		} else if n < 0x10000 {
			self.0.push(opcodes::all::OP_PUSHDATA2.into_u8());
			self.0.push((n % 0x100) as u8);
			self.0.push((n / 0x100) as u8);
		} else if n < 0x100000000 {
			self.0.push(opcodes::all::OP_PUSHDATA4.into_u8());
			self.0.push((n % 0x100) as u8);
			self.0.push(((n / 0x100) % 0x100) as u8);
			self.0.push(((n / 0x10000) % 0x100) as u8);
			self.0.push((n / 0x1000000) as u8);
		}
	}
}

impl fmt::Display for ScriptType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			ScriptType::P2SH => write!(f, "scripthash"),
			ScriptType::P2PKH => write!(f, "pubkeyhash"),
			ScriptType::Custom => write!(f, "non-standard"),
		}
    }
}

#[cfg(test)]
mod tests {
    use crate::script::ScriptPubKey;
    use crate::txio::Decodable;
    use crate::{Deserialize, Serialize};
    use super::Script;

    #[test]
	fn decode_script_1() {
		let raw_script = "76a91414011f7254d96b819c76986c277d115efce6f7b58763ac67210394854aa6eab5b2a\
		8122cc726e9dded053a2184d88256816826d6231c068d4a5b7c820120876475527c21030d417a46946384f88d5f\
		3337267c5e579765875dc4daca813e21734b140639e752ae67a914b43e1b38138a41b37f7cd9a1d274bc63e3a9b\
		5d188ac6868".to_string();
		let bytes = raw_script.decode_hex_be().expect("shouldn't fail");

		let script = match ScriptPubKey::decode_raw(bytes) {
			Ok(s) => s,
			Err(e) => panic!("{}", e)
		};

		assert_eq!(script.as_hex(), raw_script);
		assert_eq!(script.as_asm(), "OP_DUP OP_HASH160 14011f7254d96b819c76986c277d115efce6f7b5 \
		OP_EQUAL OP_IF OP_CHECKSIG OP_ELSE 0394854aa6eab5b2a8122cc726e9dded053a2184d88256816826d623\
		1c068d4a5b OP_SWAP OP_SIZE 32 OP_EQUAL OP_NOTIF OP_DROP 2 OP_SWAP 030d417a46946384f88d5f333\
		7267c5e579765875dc4daca813e21734b140639e7 2 OP_CHECKMULTISIG OP_ELSE OP_HASH160 b43e1b38138\
		a41b37f7cd9a1d274bc63e3a9b5d1 OP_EQUALVERIFY OP_CHECKSIG OP_ENDIF OP_ENDIF".to_string())
	}

    #[test]
	fn decode_script_2() {
		let raw_script = "a820affb7035b385c7e8608d209498cd85c60eddadf4e2e50356f601289198219e7387".to_string();
		let bytes = raw_script.decode_hex_be().expect("shouldn't fail");

		let script = match ScriptPubKey::decode_raw(bytes) {
			Ok(s) => s,
			Err(e) => panic!("{}", e)
		};

		assert_eq!(script.as_hex(), raw_script);
		assert_eq!(script.as_asm(), "OP_SHA256 affb7035b385c7e8608d209498cd85c60eddadf4e2e50356f601289198219e73 OP_EQUAL".to_string())
	}

    #[test]
	fn decode_script_3() {
		let raw_script = "5121022afc20bf379bc96a2f4e9e63ffceb8652b2b6a097f63fbee6ecec2a49a48010e210\
		3a767c7221e9f15f870f1ad9311f5ab937d79fcaeee15bb2c722bca515581b4c052ae".to_string();
		let bytes = raw_script.decode_hex_be().expect("shouldn't fail");

		let script = match ScriptPubKey::decode_raw(bytes) {
			Ok(s) => s,
			Err(e) => panic!("{}", e)
		};

		assert_eq!(script.as_hex(), raw_script);
		assert_eq!(script.as_asm(), "1 022afc20bf379bc96a2f4e9e63ffceb8652b2b6a097f63fbee6ecec2a49a\
		48010e 03a767c7221e9f15f870f1ad9311f5ab937d79fcaeee15bb2c722bca515581b4c0 2 OP_CHECKMULTISIG".to_string())
	}
}
