/// Code help from https://github.com/rust-bitcoin/rust-bitcoin/blob/master/src/blockdata/script.rs

use std::error::Error;
use std::io::{BufRead, Cursor, self, Seek, Read};
use crate::{Serialize, txio, Deserialize};
use serde_json::Value;

#[derive(Debug, PartialEq, Clone)]
pub struct Transaction {
	version: u32,
	flag: Option<u16>,
	in_counter: u64, // varint -> byte size 1-9
	inputs: Vec<Input>,
	out_counter: u64, // varint -> byte size 1-9
	outputs: Vec<Output>,
	lock_time: u32,
	extra_info: Option<ExtraInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Input {
	/// Previous transaction hash. Doubled SHA256-hashed.
	previous_tx: String,
	/// Index of an output
	tx_index: u32,
	/// <unlocking script> <locking script>
	script_sig: String,
	/// Relative locktime of the input
	sequence: String,
	/// Previous output
	prevout: Option<Output>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Output {
	amount: u64,
	script_pub_key: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExtraInfo {
	miner_fee: u64,
	tx_size: u64,
}

/// Build a transaction piece by piece
// struct TransactionBuilder(Vec<u8>);

impl Serialize for Transaction {
	fn new<R: BufRead>(mut reader: R) -> Self {
		println!("1. Version? (enter 1 or 2): ");
		let version = txio::user_read_u32(&mut reader);
		// println!("2. Segwit? (enter true or false)");
		// let flag = if txio::user_read_bool() { Some(1) } else { None };
		let flag = None;
		println!("2. Number of inputs?: ");
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

		println!("4. Number of outputs?: ");
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

		println!("6. Locktime (in decimal): ");
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

	fn as_hex(&self) -> String {
		let mut stream = Cursor::new(Vec::new());
		txio::write_u32_le(&mut stream, self.version);
		if let Some(flag) = self.flag {
			txio::write_u16_le(&mut stream, flag);
		}

		txio::write_varint(&mut stream, self.in_counter);

		for input in &self.inputs {
			txio::write_hex_le(&mut stream, input.previous_tx.clone(), false);
			txio::write_u32_le(&mut stream, input.tx_index);
			txio::write_hex_be(&mut stream, input.script_sig.clone(), true);
			txio::write_hex_le(&mut stream, input.sequence.clone(), false);
		}

		txio::write_varint(&mut stream, self.out_counter);

		for output in &self.outputs {
			txio::write_u64_le(&mut stream, output.amount);
			txio::write_hex_be(&mut stream, output.script_pub_key.clone(), true);
		}

		txio::write_u32_le(&mut stream, self.lock_time);

		stream.seek(io::SeekFrom::Start(0)).expect("Stream is empty?");

		let mut raw_transaction: Vec<u8> = Vec::new();
		stream.read_to_end(&mut raw_transaction).expect("Couldn't read till end");
		txio::encode_hex_be(&raw_transaction)
	}
}

impl Deserialize for Transaction {
	fn from_raw(data: String) -> Result<Self, Box<dyn Error>> {
		let raw_transaction = match serde_json::from_str::<Value>(&data) {
			Ok(d) => d["result"].to_string(),
			Err(_) => data 
		};
		println!("raw transaction: {}", raw_transaction);
		println!("-------------------");

		// convert to bytes
		let result: Vec<u8> = txio::decode_hex_be(&raw_transaction)?;
		let mut stream = Cursor::new(result);

		// version: always 4 bytes long
		let version = txio::read_u32_le(&mut stream);

		// optional, always 0001 if present
		let mut flag = Some(txio::read_u16_be(&mut stream));
		if flag != Some(1) {
			txio::unread(&mut stream, -2);
			flag = None
		}

		// number of inputs
		let in_counter = txio::read_compact_size(&mut stream);

		let mut inputs: Vec<Input> = Vec::new();
		for _ in 0..in_counter {

			let previous_tx = txio::read_hex256_le(&mut stream);
			let tx_index = txio::read_u32_le(&mut stream);
			// question: why are there n extra bytes in script_sig? in/out_script_length specifies it
			let in_script_length = txio::read_compact_size(&mut stream);
			let script_sig= txio::read_hex_var_be(&mut stream, in_script_length);
			let sequence = txio::read_hex32_le(&mut stream);
			let prevout = match get_prevout(&previous_tx, tx_index) {
				Ok(output) => Some(output),
				Err(_) => None
			};

			let input = Input {
				previous_tx,
				tx_index,
				script_sig,
				sequence,
				prevout
			};

			inputs.push(input);
		}


		// number of outputs
		let out_counter = txio::read_compact_size(&mut stream);
		assert_ne!(out_counter, 0);

		let mut outputs: Vec<Output> = Vec::new();
		for _ in 0..out_counter {

			let amount = txio::read_u64_le(&mut stream);
			let out_script_length = txio::read_compact_size(&mut stream);
			let script_pub_key = txio::read_hex_var_be(&mut stream, out_script_length);

			let output = Output {
				amount,
				script_pub_key,
			};

			outputs.push(output);
		}

		// list of witnesses
		if flag.is_some() {}

		// always 4 bytes long
		let lock_time = txio::read_u32_le(&mut stream);

		let extra_info: Option<ExtraInfo>;

		if inputs.iter().all(|x| x.prevout.is_some()) {
			// not sure if the x.prevout.to_owned().unwrap() is the best solution here.
			let total_input_amount = inputs.iter().fold(0, |acc, x| acc + x.prevout.to_owned().unwrap().amount);
			let total_output_amount = outputs.iter().fold(0, |acc, x| acc + x.amount);
			assert!(total_output_amount <= total_input_amount);
			let miner_fee = total_input_amount - total_output_amount;

			let tx_size = stream.position();

			extra_info = Some(ExtraInfo { 
				miner_fee,
				tx_size 
			});

		} else {
			extra_info = None;
		}

		let transaction = Transaction {
			version,
			flag,
			in_counter,
			inputs,
			out_counter,
			outputs,
			lock_time,
			extra_info,
		};

		Ok(transaction)
	}
}

// TODO: where should I place this? Ideally private inside Transaction impl since it's used there.
fn get_prevout(previous_tx: &str, index: u32) -> Result<Output, Box<dyn Error>> {
	let endpoint = "https://blockstream.info/api/tx/".to_string() + previous_tx;
	let response = reqwest::blocking::get(endpoint)?.json::<serde_json::Value>()?;

	let prevouts = response["vout"].as_array().unwrap();

	let output = Output {
		amount: prevouts[index as usize]["value"].as_u64().unwrap_or(0),
		script_pub_key: prevouts[index as usize]["scriptpubkey"].to_string().replace("\"", ""),
	};

	Ok(output)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
	use std::io::prelude::*;

use crate::{Serialize, Deserialize};
use crate::transaction::{Input, Output, Transaction};

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
		let tx = match Transaction::from_raw(raw_transaction) {
			Ok(t) => t,
			Err(e) => panic!("{}", e)
		};

		assert_eq!(transaction, tx);
	}
}
