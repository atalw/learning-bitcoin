use std::error::Error;
use std::io::{BufRead, Cursor};

use crate::{Serialize, opcodes, Deserialize, txio};
use std::fmt;


// Why box? https://doc.rust-lang.org/book/ch15-01-box.html
pub struct Script(Box<[u8]>);

/// Build the script piece by piece
struct ScriptBuilder(Vec<u8>);

impl Serialize for Script {
	fn new<R: BufRead>(reader: R) -> Self {
		Script::new_p2sh(&[0; 32])
	}

	fn as_hex(&self) -> String {
		format!("{:02x}", self)
	}
}

impl Script {
	fn new_p2pkh(script_hash: &[u8]) -> Self {
		let mut script_builder = ScriptBuilder::new();
		script_builder.push_opcode(opcodes::all::OP_DUP);
		script_builder.push_opcode(opcodes::all::OP_HASH160);
		script_builder.push_script_hash(script_hash);
		script_builder.push_opcode(opcodes::all::OP_EQUALVERIFY);
		script_builder.push_opcode(opcodes::all::OP_CHECKSIG);
		script_builder.into_script()
	}

	fn new_p2sh(script_hash: &[u8]) -> Self {
		let mut script_builder = ScriptBuilder::new();
		script_builder.push_opcode(opcodes::all::OP_HASH160);
		script_builder.push_script_hash(script_hash);
		script_builder.push_opcode(opcodes::all::OP_EQUAL);
		script_builder.into_script()
	}
}

impl Deserialize for Script {
	fn decode_raw(data: String) -> Result<Self, Box<dyn Error>> {
		let data = txio::decode_hex_be(&data).expect("uho ho");
		let len = data.len();
		let mut stream = Cursor::new(data);

		let mut script_builder = ScriptBuilder::new();

		while (stream.position() as usize) < len {
			let b = txio::read_u8_le(&mut stream);
			let opcode = opcodes::All::from(b);

			// not sure if this is the correct condition
			if opcode.code == opcodes::all::OP_PUSHBYTES_1.into_u8() {
				let size = txio::read_u8_le(&mut stream);
				script_builder.push_size(size);
			} else if opcode.code > opcodes::all::OP_PUSHBYTES_1.into_u8() && opcode.code <= opcodes::all::OP_PUSHBYTES_75.into_u8() {
				let len = opcode.code;
				let script = txio::read_hex_var_be(&mut stream, len as u64);
				let script_hex = txio::decode_hex_be(&script).expect("script incorrect");
				script_builder.push_script_hash(&script_hex);
			} else if opcode.code >= opcodes::all::OP_PUSHNUM_1.into_u8() && 
				opcode.code <= opcodes::all::OP_PUSHNUM_15.into_u8() {
					// let num = 1 + opcode.code - opcodes::all::OP_PUSHNUM_1.code;
					script_builder.push_opcode(opcode);
			} else {
				script_builder.push_opcode(opcode);
			}
		}

		Ok(script_builder.into_script())
	}

	// I don't like that this code is repeated. How do I reuse?
	fn as_asm(&self) -> String { 
		let data = txio::decode_hex_be(&self.as_hex()).expect("uho ho");
		let len = data.len();
		let mut stream = Cursor::new(data);

		let mut parsed = "".to_string();

		while (stream.position() as usize) < len {
			let b = txio::read_u8_le(&mut stream);
			let opcode = opcodes::All::from(b);

			// not sure if this is the correct condition
			if opcode.code == opcodes::all::OP_PUSHBYTES_1.into_u8() {
				let size = txio::read_u8_le(&mut stream);
				parsed.push_str(&format!("{}", size));
				parsed.push_str(" ");
			} else if opcode.code > opcodes::all::OP_PUSHBYTES_1.into_u8() && opcode.code <= opcodes::all::OP_PUSHBYTES_75.into_u8() {
				let len = opcode.code;
				let script = txio::read_hex_var_be(&mut stream, len as u64);
				parsed.push_str(&script);
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

impl fmt::Debug for Script {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_str("Script(")?;
		write!(f, "{}", self.as_asm())?;
		f.write_str(")")
	}
}

impl fmt::LowerHex for Script {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for &ch in self.0.iter() {
            write!(f, "{:02x}", ch)?;
        }
        Ok(())
    }
}



impl ScriptBuilder {
	fn new() -> Self {
		ScriptBuilder(vec![])	
	}

	fn into_script(&self) -> Script {
		Script(self.0.clone().into_boxed_slice())
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

#[cfg(test)]
mod tests {
    use crate::Deserialize;
    use super::Script;

    #[test]
	fn decode_script_asm() {
		let raw_script = "76a91414011f7254d96b819c76986c277d115efce6f7b58763ac67210394854aa6eab5b2a8122cc726e9dded053a2184d88256816826d6231c068d4a5b7c820120876475527c21030d417a46946384f88d5f3337267c5e579765875dc4daca813e21734b140639e752ae67a914b43e1b38138a41b37f7cd9a1d274bc63e3a9b5d188ac6868".to_string();

		let script = match Script::decode_raw(raw_script) {
			Ok(s) => s,
			Err(e) => panic!("{}", e)
		};

		assert_eq!(script.as_asm(), "OP_DUP OP_HASH160 14011f7254d96b819c76986c277d115efce6f7b5 OP_EQUAL OP_IF OP_CHECKSIG OP_ELSE 0394854aa6eab5b2a8122cc726e9dded053a2184d88256816826d6231c068d4a5b OP_SWAP OP_SIZE 32 OP_EQUAL OP_NOTIF OP_DROP 2 OP_SWAP 030d417a46946384f88d5f3337267c5e579765875dc4daca813e21734b140639e7 2 OP_CHECKMULTISIG OP_ELSE OP_HASH160 b43e1b38138a41b37f7cd9a1d274bc63e3a9b5d1 OP_EQUALVERIFY OP_CHECKSIG OP_ENDIF OP_ENDIF".to_string())
	}
}
