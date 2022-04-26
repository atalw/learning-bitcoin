/// Decode
/// Responsible for parsing a transaction and making it human readable
use std::error::Error;
use std::io::Cursor;

use serde_json::Value;

use crate::{txio, Transaction, Input, Output, ExtraInfo, Script, opcodes};

impl Script {
	pub fn asm(from: &str) -> String {
		let data = txio::decode_hex(from).expect("uh oh");
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

pub fn parse_raw_data(data: String) -> Result<Transaction, Box<dyn Error>> {
	let d: Value = serde_json::from_str(&data)?;
	println!("raw transaction: {}", d["result"]);
	println!("-------------------");

	// convert to bytes
	let result: Vec<u8> = txio::decode_hex(d["result"].as_str().unwrap())?;
	let mut stream = Cursor::new(result.clone());

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
			Ok(output) => output,
			Err(e) => panic!("{}", e)
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

	// List of witnesses
	if flag.is_some() {}

	// always 4 bytes long
	let lock_time = txio::read_u32_le(&mut stream);

	let total_input_amount = inputs.iter().fold(0, |acc, x| acc + x.prevout.amount);
	let total_output_amount = outputs.iter().fold(0, |acc, x| acc + x.amount);
	assert!(total_output_amount <= total_input_amount);
	let miner_fee = total_input_amount - total_output_amount;

	let tx_size = stream.position();


	let extra_info = ExtraInfo { 
		miner_fee,
		tx_size 
	};

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
