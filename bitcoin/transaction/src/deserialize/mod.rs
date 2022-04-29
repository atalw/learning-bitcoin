/// Decode
/// Responsible for parsing a transaction and making it human readable
use std::io::Cursor;


use crate::{txio, Script, opcodes};

impl Script {
	pub fn asm(from: &str) -> String {
		let data = txio::decode_hex_be(from).expect("uh oh");
		let len = data.len();
		let mut stream = Cursor::new(data);

		let mut parsed = "".to_string();

		while (stream.position() as usize) < len - 1 {
			let b = txio::read_u8_le(&mut stream);
			let opcode = opcodes::All::from(b);

			if opcode.code <= opcodes::all::OP_PUSHBYTES_75.into_u8() {
				let len = opcode.code;
				let script = txio::read_hex_var_be(&mut stream, len as u64);
				parsed.push_str(&script);
				parsed.push_str(" ");
			} else if opcode.code >= opcodes::all::OP_PUSHNUM_1.into_u8() && 
				opcode.code <= opcodes::all::OP_PUSHNUM_15.into_u8() {
					let num = 1 + opcode.code - opcodes::all::OP_PUSHNUM_1.code;
					parsed.push_str(&format!("{} ", num));
			} else {
				parsed.push_str(&format!("{:02x?} ", opcode));
			}
		}

		parsed.to_string()
	}

}
