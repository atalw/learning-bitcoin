use std::io::{Write, Cursor, BufRead, Seek, Read};
/// Responsible for creating a transaction and making it Bitcoin readable
/// Code help from https://github.com/rust-bitcoin/rust-bitcoin/blob/master/src/blockdata/script.rs

use std::{fmt, io};
use crate::{opcodes, Script, Transaction, txio, Input, Output};

use ripemd::Ripemd160;
use sha2::{Sha256, Digest};

/// Build a transaction piece by piece
struct TransactionBuilder(Vec<u8>);

/// Build the script piece by piece
struct ScriptBuilder(Vec<u8>);

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

impl Transaction {
	/// Create new transaction from arguments provided by user
	pub fn new<R>(mut reader: R) -> Self where R: BufRead {
		print!("1. Version? (enter 1 or 2): ");
		io::stdout().flush().unwrap();
		let version = txio::user_read_u32(&mut reader);
		// println!("2. Segwit? (enter true or false)");
		// let flag = if txio::user_read_bool() { Some(1) } else { None };
		let flag = None;
		print!("2. Number of inputs?: ");
		io::stdout().flush().unwrap();
		let in_counter = txio::user_read_u64(&mut reader);
		let mut inputs = Vec::new();
		for i in 0..in_counter {
			println!("3. Enter input {}:", i);
			println!("---- Previous transaction hex:");
			let previous_tx = txio::user_read_hex(&mut reader, Some(32));
			println!("---- Output index:");
			let tx_index = txio::user_read_u32(&mut reader);
			println!("---- Script_sig:");
			let script_sig = txio::user_read_hex(&mut reader, None);
			println!("---- Sequence (in hex):");
			let sequence = txio::user_read_hex(&mut reader, Some(4));
			let prevout = None;

			inputs.push(Input {
				previous_tx,
				tx_index,
				script_sig,
				sequence,
				prevout,
			});
		}

		print!("4. Number of outputs?: ");
		io::stdout().flush().unwrap();
		let out_counter = txio::user_read_u64(&mut reader);
		let mut outputs = Vec::new();
		for i in 0..out_counter {
			println!("5. Enter output {}:", i);
			println!("---- Amount (in sats):");
			let amount = txio::user_read_u64(&mut reader);
			println!("---- Script pubkey:");
			let script_pub_key = txio::user_read_hex(&mut reader, None);

			outputs.push(Output {
				amount,
				script_pub_key,
			});
		}

		print!("6. Locktime (in decimal): ");
		io::stdout().flush().unwrap();
		let lock_time = txio::user_read_u32(&mut reader);
		let extra_info = None;

		Transaction { 
			version,
			flag,
			in_counter,
			inputs,
			out_counter,
			outputs,
			lock_time,
			extra_info,
		}
	}

	pub fn as_hex(self) -> String {
		let mut stream = Cursor::new(Vec::new());
		txio::write_u32_le(&mut stream, self.version);
		if let Some(flag) = self.flag {
			txio::write_u16_le(&mut stream, flag);
		}

		txio::write_varint(&mut stream, self.in_counter);

		for input in self.inputs {
			txio::write_hex_le(&mut stream, input.previous_tx, false);
			txio::write_u32_le(&mut stream, input.tx_index);
			txio::write_hex_be(&mut stream, input.script_sig, true);
			txio::write_hex_le(&mut stream, input.sequence, false);
		}

		txio::write_varint(&mut stream, self.out_counter);

		for output in self.outputs {
			txio::write_u64_le(&mut stream, output.amount);
			txio::write_hex_be(&mut stream, output.script_pub_key, true);
		}

		txio::write_u32_le(&mut stream, self.lock_time);

		stream.seek(io::SeekFrom::Start(0)).expect("Stream is empty?");

		let mut raw_transaction: Vec<u8> = Vec::new();
		stream.read_to_end(&mut raw_transaction).expect("Couldn't read till end");
		txio::encode_hex_be(&raw_transaction)
	}
}

impl Script {
	pub fn new_p2pkh(script_hash: &[u8]) -> Self {
		ScriptBuilder::new()
			.push_opcode(opcodes::all::OP_DUP)
			.push_opcode(opcodes::all::OP_HASH160)
			.push_script_hash(script_hash)
			.push_opcode(opcodes::all::OP_EQUALVERIFY)
			.push_opcode(opcodes::all::OP_CHECKSIG)
			.into_script()
	}

	pub fn new_p2sh(script_hash: &[u8]) -> Self {
		ScriptBuilder::new()
			.push_opcode(opcodes::all::OP_HASH160)
			.push_script_hash(script_hash)
			.push_opcode(opcodes::all::OP_EQUAL)
			.into_script()
	}
}

impl fmt::Debug for Script {
	// fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
	//     f.write_str("Script(")?;
	//     // self.fmt_asm(f)?;
	//     // write!(f, "{:02x}", self.0.as_ref());
	//     f.write_str(")")
	// }
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for &ch in self.0.iter() {
            write!(f, "{:02x}", ch)?;
        }
        Ok(())
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

	fn into_script(self) -> Script {
		Script(self.0.into_boxed_slice())
	}

	fn push_opcode(mut self, opcode: opcodes::All) -> Self {
		self.0.push(opcode.into_u8());
		self	
	}

	fn push_script_hash(mut self, script_hash: &[u8]) -> Self {
		self.push_var_int(script_hash.len());
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
	fn push_var_int(&mut self, n: usize) {
		if n < opcodes::all::OP_PUSHDATA1.into_u8() as usize {
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
    use std::io::Cursor;
	use std::io::prelude::*;

    use crate::{Transaction, Input, Output, deserialize};

    #[test]
    fn transaction_pre_segwit() {
		let mut stream = Cursor::new(Vec::new());

		stream.write(b"1").expect("uh oh"); // version
		stream.write(b"\n").expect("uh oh");
		// stream.write(b"false").expect("uh oh"); // segwit flag
		// stream.write(b"\n").expect("uh oh");
		stream.write(b"1").expect("uh oh");
		stream.write(b"\n").expect("uh oh");
		stream.write(b"656aa8c5894c179b2745fa8a0fb68cb10688daa7389fd47900a055cc2526cb5d").expect("uh oh");
		stream.write(b"\n").expect("uh oh");
		stream.write(b"0").expect("uh oh");
		stream.write(b"\n").expect("uh oh");
		stream.write(b"76a91488fed7b8154069b5d2ace12fa4b7f96ab73d59df88ac").expect("uh oh");
		stream.write(b"\n").expect("uh oh");
		stream.write(b"ffffffff").expect("uh oh");
		stream.write(b"\n").expect("uh oh");
		stream.write(b"1").expect("uh oh");
		stream.write(b"\n").expect("uh oh");
		stream.write(b"1000").expect("uh oh");
		stream.write(b"\n").expect("uh oh");
		stream.write(b"abcdef").expect("uh oh");
		stream.write(b"\n").expect("uh oh");
		stream.write(b"0").expect("uh oh");
		stream.write(b"\n").expect("uh oh");

		stream.seek(std::io::SeekFrom::Start(0)).expect("unable to seek");

		let inputs = vec![Input {
			previous_tx: "656aa8c5894c179b2745fa8a0fb68cb10688daa7389fd47900a055cc2526cb5d".to_string(),
			tx_index: 0,
			script_sig: "76a91488fed7b8154069b5d2ace12fa4b7f96ab73d59df88ac".to_string(),
			sequence: "ffffffff".to_string(),
			prevout: None,
		}];

		let outputs = vec![Output {
			amount: 1000,
			script_pub_key: "abcdef".to_string(),
		}];
		
		let transaction = Transaction {
			version: 1,
			flag: None,
			in_counter: 1, // varint -> byte size 1-9
			inputs,
			out_counter: 1, // varint -> byte size 1-9
			outputs,
			lock_time: 0,
			extra_info: None,
		};

		assert_eq!(Transaction::new(stream), transaction);
    }

	#[test]
	fn transaction_pre_segwit_hex() {
		let inputs = vec![Input {
			previous_tx: "656aa8c5894c179b2745fa8a0fb68cb10688daa7389fd47900a055cc2526cb5d".to_string(),
			tx_index: 0,
			script_sig: "76a91488fed7b8154069b5d2ace12fa4b7f96ab73d59df88ac".to_string(),
			sequence: "ffffffff".to_string(),
			prevout: None,
		}];

		let outputs = vec![Output {
			amount: 1000,
			script_pub_key: "abcdef".to_string(),
		}];
		
		let transaction = Transaction {
			version: 1,
			flag: None,
			in_counter: 1, // varint -> byte size 1-9
			inputs,
			out_counter: 1, // varint -> byte size 1-9
			outputs,
			lock_time: 0,
			extra_info: None,
		};

		let raw_transaction = transaction.clone().as_hex();
		assert_eq!(raw_transaction, "01000000015dcb2625cc55a00079d49f38a7da8806b18cb60f8afa45279b174c89c5a86a65000000001976a91488fed7b8154069b5d2ace12fa4b7f96ab73d59df88acffffffff01e80300000000000003abcdef00000000".to_string());

		// round trip
		let tx = match deserialize::parse_raw_data(raw_transaction) {
			Ok(t) => t,
			Err(e) => panic!("{}", e)
		};

		assert_eq!(transaction, tx);
	}
}

