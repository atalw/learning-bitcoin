/// Code help from https://github.com/rust-bitcoin/rust-bitcoin/blob/master/src/blockdata/script.rs

use std::error::Error;
use std::io::{BufRead, Cursor, self, Seek, Read};
use crate::script::Script;
use crate::txio::{Encodable, Decodable, HexBytes};
use crate::{Serialize, txio, Deserialize};
use serde_json::Value;
use derivative::Derivative;

#[derive(Derivative)]
#[derivative(Debug, PartialEq)]
pub struct Transaction {
	version: u32,
	flag: Option<u16>,
	in_counter: u64, // varint -> byte size 1-9
	inputs: Vec<Input>,
	out_counter: u64, // varint -> byte size 1-9
	outputs: Vec<Output>,
	witness_data: Option<Vec<WitnessStack>>,
	lock_time: u32,
	#[derivative(PartialEq="ignore")]
	extra_info: Option<ExtraInfo>,
}

#[derive(Derivative)]
#[derivative(Debug, PartialEq)]
pub struct Input {
	/// Previous transaction hash. Doubled SHA256-hashed.
	previous_tx: String,
	/// Index of an output
	tx_index: u32,
	/// <signature> <original script>
	script_sig: Script,
	/// Relative locktime of the input
	sequence: String,
	/// Previous output
	#[derivative(PartialEq="ignore")]
	prevout: Option<Output>,
}

#[derive(Debug, PartialEq)]
pub struct Output {
	amount: u64,
	script_pub_key: Script,
}

#[derive(Debug, PartialEq)]
pub struct WitnessStack(Vec<String>);

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
		println!("2. Segwit? (enter true or false)");
		let flag = if txio::user_read_bool(&mut reader) { Some(1) } else { None };
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
			let script_sig = Script::new(&mut reader);
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
			let script_pub_key = Script::new(&mut reader);

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
			witness_data: None,
			lock_time,
			extra_info,
		}
	}

	fn as_hex(&self) -> String {
		let mut stream = Cursor::new(Vec::new());
		txio::write_u32_le(&mut stream, self.version);
		if let Some(flag) = self.flag {
			txio::write_u16_be(&mut stream, flag);
		}

		txio::write_varint(&mut stream, self.in_counter);

		for input in &self.inputs {
			txio::write_hex_le(&mut stream, input.previous_tx.clone(), false);
			txio::write_u32_le(&mut stream, input.tx_index);
			txio::write_hex_be(&mut stream, input.script_sig.as_hex(), true);
			txio::write_hex_le(&mut stream, input.sequence.clone(), false);
		}

		txio::write_varint(&mut stream, self.out_counter);

		for output in &self.outputs {
			txio::write_u64_le(&mut stream, output.amount);
			txio::write_hex_be(&mut stream, output.script_pub_key.as_hex(), true);
		}

		if let Some(witness_data) = self.witness_data.as_ref() {
			for witnesses in witness_data {
				txio::write_varint(&mut stream, witnesses.0.len() as u64);
				for w in &witnesses.0 {
					txio::write_hex_be(&mut stream, w.to_string(), true)
				}
			}
		}

		txio::write_u32_le(&mut stream, self.lock_time);

		stream.seek(io::SeekFrom::Start(0)).expect("Stream is empty?");

		let mut raw_transaction: Vec<u8> = Vec::new();
		stream.read_to_end(&mut raw_transaction).expect("Couldn't read till end");
		raw_transaction.encode_hex_be()
		// txio::encode_hex_be(&raw_transaction)
	}

	fn as_bytes(&self) -> &[u8] {
		todo!()
	}
}

impl Deserialize for Transaction {
	fn decode_raw<R: BufRead>(reader: R) -> Result<Self, Box<dyn Error>> {
		println!("Enter a raw transaction hex");
		let hex = txio::user_read_hex(reader, None);

		let raw_transaction = match serde_json::from_str::<Value>(&hex) {
			Ok(d) => d["result"].to_string(),
			Err(_) => hex 
		};
		println!("raw transaction: {}", raw_transaction);
		println!("-------------------");

		// convert to bytes
		// let result: Box<[u8]> = txio::decode_hex_be(&raw_transaction)?;
		let data: HexBytes = raw_transaction.decode_hex_be()?;
		let mut stream = Cursor::new(data);

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
		assert_ne!(in_counter, 0);

		let mut inputs: Vec<Input> = Vec::new();
		println!("Number of inputs {} {}", in_counter, stream.position());
		for _ in 0..in_counter {
			let previous_tx = txio::read_hex256_le(&mut stream);
			let tx_index = txio::read_u32_le(&mut stream);
			// question: why are there n extra bytes in script_sig? in/out_script_length specifies it
			let in_script_length = txio::read_compact_size(&mut stream);
			let script_sig: Script;
			if in_script_length == 0 {
				script_sig = Script::from("00");
			} else {
				script_sig = Script(txio::read_hex_var_be(&mut stream, in_script_length));
			}
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
			let script_pub_key = Script(txio::read_hex_var_be(&mut stream, out_script_length));

			let output = Output {
				amount,
				script_pub_key,
			};

			outputs.push(output);
		}

		// list of witnesses
		let witness_data: Option<Vec<WitnessStack>>;
		if flag.is_some() {
			let mut _witness_data = Vec::new();
			// number of witnesses = number of inputs
			for i in 0..in_counter {
				// If a txin is not associated with any witness data, its corresponding witness 
				// field is an exact 0x00, indicating that the number of witness stack items is zero.
				if inputs[i as usize].script_sig == Script::from("00") { continue }
				let mut stack = WitnessStack(Vec::new());
				let stack_count = txio::read_compact_size(&mut stream);
				for _ in 0..stack_count {
					let length = txio::read_compact_size(&mut stream);
					// let witness = txio::encode_hex_be(&txio::read_hex_var_be(&mut stream, length));
					let witness = txio::read_hex_var_be(&mut stream, length).encode_hex_be();
					stack.0.push(witness);
				}
				_witness_data.push(stack);
			}
			witness_data = Some(_witness_data);
		} else {
			witness_data = None;
		}

		// always 4 bytes long
		let lock_time = txio::read_u32_le(&mut stream);

		let extra_info: Option<ExtraInfo>;

		if inputs.iter().all(|x| x.prevout.is_some()) {
			// not sure if the x.prevout.to_owned().unwrap() is the best solution here.
			let total_input_amount = inputs.iter().fold(0, |acc, x| acc + x.prevout.as_ref().unwrap().amount);
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
			witness_data,
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

	let amount = prevouts[index as usize]["value"].as_u64().unwrap_or(0); 
	let hex = prevouts[index as usize]["scriptpubkey"].to_string().replace("\"", "");
	// let script_pub_key = Script(txio::decode_hex_be(&hex)?);
	let script_pub_key = Script(hex.decode_hex_be()?);

	let output = Output {
		amount,
		script_pub_key,
	};

	Ok(output)
}

#[cfg(test)]
mod tests {
	use std::io::{Cursor, Error};
	use std::io::prelude::*;
	use crate::{Serialize, Deserialize};
	use crate::transaction::{Input, Output, Transaction, WitnessStack};
	use crate::Script;

    #[test]
    fn encode_transaction_pre_segwit() -> Result<(), Error> {
		let mut stream = Cursor::new(Vec::new());

		stream.write(b"1")?; // version
		stream.write(b"\n")?;
		stream.write(b"false")?;
		stream.write(b"\n")?;
		stream.write(b"1")?;
		stream.write(b"\n")?;
		stream.write(b"656aa8c5894c179b2745fa8a0fb68cb10688daa7389fd47900a055cc2526cb5d")?;
		stream.write(b"\n")?;
		stream.write(b"0")?;
		stream.write(b"\n")?;
		stream.write(b"1")?;
		stream.write(b"\n")?;
		stream.write(b"2")?;
		stream.write(b"\n")?;
		stream.write(b"OP_DUP OP_HASH160 88fed7b8154069b5d2ace12fa4b7f96ab73d59df OP_EQUALVERIFY OP_CHECKSIG")?;
		stream.write(b"\n")?;
		stream.write(b"ffffffff")?;
		stream.write(b"\n")?;
		stream.write(b"1")?;
		stream.write(b"\n")?;
		stream.write(b"1000")?;
		stream.write(b"\n")?;
		stream.write(b"4")?;
		stream.write(b"\n")?;
		stream.write(b"abcdef")?;
		stream.write(b"\n")?;
		stream.write(b"0")?;
		stream.write(b"\n")?;

		stream.seek(std::io::SeekFrom::Start(0))?;

		let inputs = vec![Input {
			previous_tx: "656aa8c5894c179b2745fa8a0fb68cb10688daa7389fd47900a055cc2526cb5d".to_string(),
			tx_index: 0,
			script_sig: Script::from("a91430fc33f7b86c02f3edb60ea373ca5f467cf507b787"),
			sequence: "ffffffff".to_string(),
			prevout: None,
		}];
		let outputs = vec![Output {
			amount: 1000,
			script_pub_key: Script::from("abcdef"),
		}];
		let witness_data = None;
		let transaction = Transaction {
			version: 1,
			flag: None,
			in_counter: 1, // varint -> byte size 1-9
			inputs,
			out_counter: 1, // varint -> byte size 1-9
			outputs,
			witness_data,
			lock_time: 0,
			extra_info: None,
		};

		assert_eq!(Transaction::new(stream), transaction);
		Ok(())
    }

	#[test]
	fn decode_transaction_pre_segwit_1() {
		let inputs = vec![Input {
			previous_tx: "656aa8c5894c179b2745fa8a0fb68cb10688daa7389fd47900a055cc2526cb5d".to_string(),
			tx_index: 0,
			script_sig: Script::from("76a91488fed7b8154069b5d2ace12fa4b7f96ab73d59df88ac"),
			sequence: "ffffffff".to_string(),
			prevout: None,
		}];
		let outputs = vec![Output {
			amount: 1000,
			script_pub_key: Script::from("abcdef"),
		}];
		let witness_data = None;
		let transaction = Transaction {
			version: 1,
			flag: None,
			in_counter: 1,
			inputs,
			out_counter: 1,
			outputs,
			witness_data,
			lock_time: 0,
			extra_info: None,
		};

		assert_eq!(transaction.as_hex(), "01000000015dcb2625cc55a00079d49f38a7da8806b18cb60f8afa452\
		79b174c89c5a86a65000000001976a91488fed7b8154069b5d2ace12fa4b7f96ab73d59df88acffffffff01e803\
		00000000000003abcdef00000000".to_string());

		// round trip
		let mut stream = Cursor::new(Vec::new());
		stream.write(b"01000000015dcb2625cc55a00079d49f38a7da8806b18cb60f8afa45279b174c89c5a86a6500\
					 0000001976a91488fed7b8154069b5d2ace12fa4b7f96ab73d59df88acffffffff01e803000000\
					 00000003abcdef00000000").expect("uh oh");
		stream.write(b"\n").expect("uh oh");
		stream.seek(std::io::SeekFrom::Start(0)).expect("unable to seek");

		let tx = match Transaction::decode_raw(stream) {
			Ok(t) => t,
			Err(e) => panic!("{}", e)
		};

		assert_eq!(transaction, tx);
	}

	#[test]
	fn decode_transaction_pre_segwit_2() {
		let inputs = vec![Input {
			previous_tx: "0437cd7f8525ceed2324359c2d0ba26006d92d856a9c20fa0241106ee5a597c9".to_string(),
			tx_index: 0,
			script_sig: Script::from("47304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61\
			548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901"),
			sequence: "ffffffff".to_string(),
			prevout: None,
		}];

		let outputs = vec![
			Output {
				amount: 1000000000,
				script_pub_key: Script::from("4104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd3\
				78d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac"),
			},
			Output {
				amount: 4000000000,
				script_pub_key: Script::from("410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ec\
				ad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac"),
			}
		];
		
		let witness_data = None;

		let transaction = Transaction {
			version: 1,
			flag: None,
			in_counter: 1,
			inputs,
			out_counter: 2,
			outputs,
			witness_data,
			lock_time: 0,
			extra_info: None,
		};

		let raw_tx = "0100000001c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd3704000\
		000004847304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522\
		ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901ffffffff0200ca9a3b0000000043410\
		4ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21\
		c1b7303b8a0626f1baded5c72a704f7e6cd84cac00286bee0000000043410411db93e1dcdb8a016b49840f8c53b\
		c1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f6\
		56b412a3ac00000000";

		assert_eq!(transaction.as_hex(), raw_tx.to_string());

		// round trip
		let mut stream = Cursor::new(Vec::new());
		stream.write(raw_tx.as_bytes()).expect("uh oh");
		stream.write(b"\n").expect("uh oh");
		stream.seek(std::io::SeekFrom::Start(0)).expect("unable to seek");

		let tx = match Transaction::decode_raw(stream) {
			Ok(t) => t,
			Err(e) => panic!("{}", e)
		};

		assert_eq!(transaction, tx);
	}

	#[test]
	fn decode_transaction_pre_segwit_3() {
		let inputs = vec![Input {
			previous_tx: "889c2561c6caf5f31af96162b17b196cc88a81f04f5f4a9af9052529c4f71ae1".to_string(),
			tx_index: 0,
			script_sig: Script::from("473044022045c7199ffc8069a498135b7bb2678da16e8b5d49455b4a7ace7\
			55928c9339c7a022051cbf72024cf273444640f7b993b2bf3d329124b03e6744edaed5158a30e29b8012103\
			fd9bc1e9803e739720e0f1c63e580a94656c7d0cab6cd083f0c0dfb221b90662"),
			sequence: "ffffffff".to_string(),
			prevout: None,
		}];

		let outputs = vec![
			Output {
				amount: 1400000000000,
				script_pub_key: Script::from("76a9143b9552116adcc2fbd74fad44a4da603a727c816e88ac")
			},
			Output {
				amount: 1099994980000,
				script_pub_key: Script::from("76a914f90ce447f14847e841d4d2ecc76299b5bc77166188ac"),
			}
		];
		let witness_data = None;
		let transaction = Transaction {
			version: 1,
			flag: None,
			in_counter: 1,
			inputs,
			out_counter: 2,
			outputs,
			witness_data,
			lock_time: 0,
			extra_info: None,
		};

		let raw_transaction = "0100000001e11af7c4292505f99a4a5f4ff0818ac86c197bb16261f91af3f5cac661\
		259c88000000006a473044022045c7199ffc8069a498135b7bb2678da16e8b5d49455b4a7ace755928c9339c7a0\
		22051cbf72024cf273444640f7b993b2bf3d329124b03e6744edaed5158a30e29b8012103fd9bc1e9803e739720\
		e0f1c63e580a94656c7d0cab6cd083f0c0dfb221b90662ffffffff0200b080f6450100001976a9143b9552116ad\
		cc2fbd74fad44a4da603a727c816e88aca05ecf1c000100001976a914f90ce447f14847e841d4d2ecc76299b5bc\
		77166188ac00000000";

		assert_eq!(transaction.as_hex(), raw_transaction.to_string());

		// round trip
		let mut stream = Cursor::new(Vec::new());

		stream.write(raw_transaction.as_bytes()).expect("uh oh");
		stream.write(b"\n").expect("uh oh");

		stream.seek(std::io::SeekFrom::Start(0)).expect("unable to seek");

		let tx = match Transaction::decode_raw(stream) {
			Ok(t) => t,
			Err(e) => panic!("{}", e)
		};

		assert_eq!(transaction, tx);
	}
	#[test]
	fn decode_transaction_pre_segwit_4() {
		let inputs = vec![Input {
			previous_tx: "cc526c2f5d31894c27641469bfc751910aaa08202e038b0ec6f0a9f661d3ba6d".to_string(),
			tx_index: 25,
			script_sig: Script::from("0047304402204945c3e4f824d263bb22e117a12bfff741d996d594f07551c\
			93e0fde77910d32022016c2b69daec51bd4afdd81bf90f76667dda515773b3da91174043fc7299acb530147\
			3044022053c71a4730160b20e565cb669a44b793f42d2912e84d528cf203089abcb2874402203311303cfc3\
			6b91372e47d5fa0b22104e7c25bb5a8dcccd15c423620d5700304014c69522102047464f518269c6cba42b8\
			59d28e872ef8f6bb47d93e24d5c11ac6eca8a2845721029b48417598a2d2dab54ddddfca8e1a9c8d4967002\
			180961f53a7748710c2176521036b1023b6c7ed689aaf3bc8ca9ee5c55da383ae0c44fc8b0fec91d6965dae\
			5d5e53ae"),
			sequence: "ffffffff".to_string(),
			prevout: None,
		}];

		let outputs = vec![
			Output {
				amount: 1170000,
				script_pub_key: Script::from("00141e129251311437eea493fce2a3644a5a1af8d407"),
			},
			Output {
				amount: 1930000,
				script_pub_key: Script::from("76a9140ac4423b045a0c8ed5f4fb992256ed293a313ae088ac"),
			},
			Output {
				amount: 10185620,
				script_pub_key: Script::from("a914cd38af19a803de11ddcee3a45221ed9ac491404787"),
			},
			Output {
				amount: 1519708769,
				script_pub_key: Script::from("a9143572de0bb360f212ef8813a9e012f63a7035c9c987"),
			}
		];

		let witness_data = None;
		
		let transaction = Transaction {
			version: 2,
			flag: None,
			in_counter: 1,
			inputs,
			out_counter: 4,
			outputs,
			witness_data,
			lock_time: 0,
			extra_info: None,
		};

		println!("{:#?}", transaction);

		// txid: db6e06ff6e53356cc22cd1b9b8d951ddf70dc6bb275ee76880a0b951c1c290e6
		let raw_transaction = "02000000016dbad361f6a9f0c60e8b032e2008aa0a9151c7bf691464274c89315d2f\
		6c52cc19000000fc0047304402204945c3e4f824d263bb22e117a12bfff741d996d594f07551c93e0fde77910d3\
		2022016c2b69daec51bd4afdd81bf90f76667dda515773b3da91174043fc7299acb5301473044022053c71a4730\
		160b20e565cb669a44b793f42d2912e84d528cf203089abcb2874402203311303cfc36b91372e47d5fa0b22104e\
		7c25bb5a8dcccd15c423620d5700304014c69522102047464f518269c6cba42b859d28e872ef8f6bb47d93e24d5\
		c11ac6eca8a2845721029b48417598a2d2dab54ddddfca8e1a9c8d4967002180961f53a7748710c2176521036b1\
		023b6c7ed689aaf3bc8ca9ee5c55da383ae0c44fc8b0fec91d6965dae5d5e53aeffffffff0450da110000000000\
		1600141e129251311437eea493fce2a3644a5a1af8d40710731d00000000001976a9140ac4423b045a0c8ed5f4f\
		b992256ed293a313ae088ac946b9b000000000017a914cd38af19a803de11ddcee3a45221ed9ac49140478761ea\
		945a0000000017a9143572de0bb360f212ef8813a9e012f63a7035c9c98700000000";

		assert_eq!(transaction.as_hex(), raw_transaction.to_string());

		// round trip
		let mut stream = Cursor::new(Vec::new());
		stream.write(raw_transaction.as_bytes()).expect("uh oh");
		stream.write(b"\n").expect("uh oh");
		stream.seek(std::io::SeekFrom::Start(0)).expect("unable to seek");

		let tx = match Transaction::decode_raw(stream) {
			Ok(t) => t,
			Err(e) => panic!("{}", e)
		};

		assert_eq!(transaction, tx);
	}

	#[test]
	fn decode_transaction_segwit_1() {
		let inputs = vec![
			Input {
				previous_tx: "4c13d894ad116e56d473b30774ea8577017cb51384c0104e4ee2d432b84194d1".to_string(),
				tx_index: 28,
				script_sig: Script::from("1600147c846a806f4d9e516c9fb2fe364f28eac4e3c3fc"),
				sequence: "ffffffff".to_string(),
				prevout: None,
			},
			Input {
				previous_tx: "9a71c410ee5b3c3e2949f17db6220dd0887aaae63f0ecccbee2e4241596a413d".to_string(),
				tx_index: 44,
				script_sig: Script::from("1600147c846a806f4d9e516c9fb2fe364f28eac4e3c3fc"),
				sequence: "ffffffff".to_string(),
				prevout: None,
			},
			Input {
				previous_tx: "966973ad982bb773854f87a725563067f332199f6b632dcca38127299a58224f".to_string(),
				tx_index: 0,
				script_sig: Script::from("1600147c846a806f4d9e516c9fb2fe364f28eac4e3c3fc"),
				sequence: "ffffffff".to_string(),
				prevout: None,
			},

		];

		let outputs = vec![
			Output {
				amount: 73950453,
				script_pub_key: Script::from("a9142c21151d54bd219dcc4c52e1cb38672dab8e36cc87"),
			},
			Output {
				amount: 1147480000,
				script_pub_key: Script::from("76a91439b1050dba04b1d1bc556c2dcdcb3874ba3dc11e88ac"),
			}
		];

		let witness_data = Some(vec![
			WitnessStack { 0: vec![
				"304402203ccede7995b26185574a050373cfe607f475f7d8ee6927647c496e3b45bf61a302202bd1ff\
					88c7f4ee0b6f0c98f687dff9033f770b23985f590d178b9085df58910101".to_string(),
				"03789a9d83798d4cbf688f9969a94084ee1655059e137b43492ee94dc4538790ab".to_string(),
			]},
			WitnessStack { 0: vec![
				"3045022100b46ab18056655cc56b1778fd61a56f895c2f44c97f055ea0269d991efd181fb402206d65\
					1a5fb51081cfdb247a1d489b182f41e52434d7c4575bea30d2ce3d24087d01".to_string(),
				"03789a9d83798d4cbf688f9969a94084ee1655059e137b43492ee94dc4538790ab".to_string()
			]},
			WitnessStack { 0: vec![
				"3044022069bf2ac34569565a62a1e0c12750104f494a906fefd2f2a462199c0d4bc235d902200c37ef\
					333b453966cc5e84b178ec62125cbed83e0c0df4448c0fb331efa49e5101".to_string(),
				"03789a9d83798d4cbf688f9969a94084ee1655059e137b43492ee94dc4538790ab".to_string()
			]},
		]);
		
		let transaction = Transaction {
			version: 2,
			flag: Some(1),
			in_counter: 3,
			inputs,
			out_counter: 2,
			outputs,
			witness_data,
			lock_time: 0,
			extra_info: None,
		};

		println!("{:#?}", transaction);

		// txid: 68333a10b368e0d002098827fa3f348135fb728ade74d265e6abf41dfcb60a1c
		let raw_transaction = "02000000000103d19441b832d4e24e4e10c08413b57c017785ea7407b373d4566e11\
		ad94d8134c1c000000171600147c846a806f4d9e516c9fb2fe364f28eac4e3c3fcffffffff3d416a5941422eeec\
		bcc0e3fe6aa7a88d00d22b67df149293e3c5bee10c4719a2c000000171600147c846a806f4d9e516c9fb2fe364f\
		28eac4e3c3fcffffffff4f22589a292781a3cc2d636b9f1932f367305625a7874f8573b72b98ad7369960000000\
		0171600147c846a806f4d9e516c9fb2fe364f28eac4e3c3fcffffffff02f56468040000000017a9142c21151d54\
		bd219dcc4c52e1cb38672dab8e36cc87c0276544000000001976a91439b1050dba04b1d1bc556c2dcdcb3874ba3\
		dc11e88ac0247304402203ccede7995b26185574a050373cfe607f475f7d8ee6927647c496e3b45bf61a302202b\
		d1ff88c7f4ee0b6f0c98f687dff9033f770b23985f590d178b9085df589101012103789a9d83798d4cbf688f996\
		9a94084ee1655059e137b43492ee94dc4538790ab02483045022100b46ab18056655cc56b1778fd61a56f895c2f\
		44c97f055ea0269d991efd181fb402206d651a5fb51081cfdb247a1d489b182f41e52434d7c4575bea30d2ce3d2\
		4087d012103789a9d83798d4cbf688f9969a94084ee1655059e137b43492ee94dc4538790ab02473044022069bf\
		2ac34569565a62a1e0c12750104f494a906fefd2f2a462199c0d4bc235d902200c37ef333b453966cc5e84b178e\
		c62125cbed83e0c0df4448c0fb331efa49e51012103789a9d83798d4cbf688f9969a94084ee1655059e137b4349\
		2ee94dc4538790ab00000000";

		assert_eq!(transaction.as_hex(), raw_transaction.to_string());

		// round trip
		let mut stream = Cursor::new(Vec::new());
		stream.write(raw_transaction.as_bytes()).expect("uh oh");
		stream.write(b"\n").expect("uh oh");
		stream.seek(std::io::SeekFrom::Start(0)).expect("unable to seek");

		let tx = match Transaction::decode_raw(stream) {
			Ok(t) => t,
			Err(e) => panic!("{}", e)
		};

		assert_eq!(transaction, tx);
	}
}
