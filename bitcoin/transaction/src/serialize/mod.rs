/// Responsible for creating a transaction and making it Bitcoin readable
/// Code help from https://github.com/rust-bitcoin/rust-bitcoin/blob/master/src/blockdata/script.rs

use crate::opcodes;

// Why box? https://doc.rust-lang.org/book/ch15-01-box.html
pub struct Script(Box<[u8]>);

/// Build the script piece by piece
struct Builder(Vec<u8>);

impl Script {
	pub fn new_p2sh(script_hash: &[u8]) -> Self {
		Builder::new()
			.push_opcode(opcodes::all::OP_HASH160)
			.push_script_hash(script_hash)
			.push_opcode(opcodes::all::OP_EQUAL)
			.into_script()
	}
}

impl Builder {
	pub fn new() -> Self {
		Builder(vec![])	
	}

	pub fn into_script(mut self) -> Script {
		Script(self.0.into_boxed_slice())
	}

	pub fn push_opcode(mut self, opcode: opcodes::All) -> Self {
		self.0.push(opcode.into_u8());
		self	
	}

	pub fn push_script_hash(mut self, script_hash: &[u8]) -> Self {
		self.write_var_int(script_hash.len());
		self.0.extend(script_hash.iter().cloned());
		self	
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
	fn write_var_int(mut self, n: usize) {
		if n < opcodes::all::OP_PUSHDATA1 as usize {
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
