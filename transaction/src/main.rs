use std::error::Error;use std::fmt::write;
use std::num::ParseIntError;

use serde_json::Value;

#[derive(Debug)]
struct Transaction {
	version: u32,
	flag: Option<u16>,
	in_counter: u16,
	inputs: Vec<Input>,
	out_counter: u16,
	outputs: Vec<Output>,
	lock_time: u32
}

#[derive(Debug)]
struct Input {
	previous_tx: String,
	tx_index: String,
	script_sig: String,
	sequence: String,
}

#[derive(Debug)]
struct Output {
	amount: i64,
	script_pub_key: String,
}

fn main() {
	for raw_transaction in get_raw_transactions() {
		match parse_raw_data(raw_transaction) {
			Ok(transaction) => { println!("{:#?}", transaction) },
			Err(e) => panic!("{}", e)
		}
		break
	}
}

fn parse_raw_data(data: String) -> Result<Transaction, Box<dyn Error>> {

	let d: Value = serde_json::from_str(&data)?;
	println!("raw transaction: {}", d["result"]);
	println!("-------------------");

	let result: Vec<u8> = decode_hex(d["result"].as_str().unwrap())?;

	// println!("input bytes: {:?}", result);
	// println!("-------------------");

	let mut pointer: usize = 0;

	// version: always 4 bytes long
	pointer += 4;
	let mut version_bytes = [0; 4];
	version_bytes.copy_from_slice(&result[..pointer]);
	let version = get_decimal_value_u32(&version_bytes);
	// println!("version: {:?} {} {}", version_bytes, version, u32::from_le_bytes(version_bytes));

	// optional, always 0001 if present
	let mut segwit_bytes = [0; 2];
	segwit_bytes.copy_from_slice(&result[pointer..pointer+2]);
	let mut flag: Option<u16> = None;
	if segwit_bytes[1] == 1 { 
		pointer += 2;
		flag = Some(0001);
		// println!("segwit flag: {:?}", encode_hex(&segwit_bytes));
	}


	let mut in_counter_bytes = [0; 1];
	in_counter_bytes.copy_from_slice(&result[pointer..pointer+1]);
	let in_counter = get_decimal_value_u16(&in_counter_bytes);
	assert_ne!(in_counter, 0);
	// println!("number of inputs: {:?}", in_counter);
	pointer += 1;

	let mut inputs: Vec<Input> = Vec::new();
	let mut index: usize = 0;
	for i in 0..in_counter {

		// start_index: 7
		// start_index: 7 + 64 (each input is 64 bytes)
		let mut previous_tx_bytes = [0; 32];
		let start_index = pointer + (i*64) as usize;
		let end_index = start_index + 32;
		previous_tx_bytes.copy_from_slice(&result[start_index..end_index]);
		let previous_tx = encode_hex(&previous_tx_bytes);

		let mut tx_index_bytes = [0; 4];
		let start_index = end_index;
		let end_index = start_index + 4;
		tx_index_bytes.copy_from_slice(&result[start_index..end_index]);
		let tx_index = encode_hex(&tx_index_bytes);

		// Taking length of 1 byte is wrong. Can be of length 1, 3, 5, or 9. Check spec
		// https://en.bitcoin.it/wiki/Transaction
		// https://en.bitcoin.it/wiki/Protocol_documentation#Variable_length_integer
		let mut out_script_length_bytes = [0; 1];
		let start_index = end_index;
		let end_index = start_index + 1;
		out_script_length_bytes.copy_from_slice(&result[start_index..end_index]);
		let out_script_length = get_decimal_value_usize(&out_script_length_bytes);

		let mut script_sig_bytes = vec![0; out_script_length];
		let start_index = end_index;
		let end_index = start_index + out_script_length;
		script_sig_bytes.copy_from_slice(&result[start_index..end_index]);
		let script_sig= encode_hex(&script_sig_bytes);

		let mut sequence_bytes = [0; 4];
		let start_index = end_index;
		let end_index = start_index + 4;
		index = end_index;
		sequence_bytes.copy_from_slice(&result[start_index..end_index]);
		let sequence = encode_hex(&sequence_bytes);

		let input = Input {
			previous_tx,
			tx_index,
			script_sig,
			sequence,
		};

		inputs.push(input);
	}

	// println!("vin: {:#?}", inputs);

	let mut out_counter_bytes = [0; 1];
	out_counter_bytes.copy_from_slice(&result[index..index+1]);
	let out_counter = get_decimal_value_u16(&out_counter_bytes);
	assert_ne!(out_counter, 0);
	// println!("number of outputs: {:?}", out_counter);

	let mut outputs: Vec<Output> = Vec::new();
	let mut index = index + 1;
	let mut output_size;

	for _ in 0..out_counter {

		output_size = 0;

		// start_index: index + 32 (each output is 32 bytes)
		let mut amount_bytes= [0; 8];
		let start_index = index + output_size as usize;
		let end_index = start_index + 8;
		amount_bytes.copy_from_slice(&result[start_index..end_index]);
		let amount = get_decimal_value_i64(&amount_bytes);

		output_size += 8;

		// question:   why are there 2 extra bytes? out_script_length specifies it
		// output address 1: a9142c21151d54bd219dcc4c52e1cb38672dab8e36cc87
		// output address 2: 76a91439b1050dba04b1d1bc556c2dcdcb3874ba3dc11e88ac
		// Taking length of 1 byte is wrong. Can be of length 1, 3, 5, or 9. Check spec
		// https://en.bitcoin.it/wiki/Transaction
		// https://en.bitcoin.it/wiki/Protocol_documentation#Variable_length_integer
		// fc -> 0-252
		// fd -> 0000 (253 + 2 bytes)
		// fe -> 0000 0000 (254 + 4 bytes)
		// ff -> 0000 0000 0000 0000 (255 + 8 bytes)
		// check bitcoin/src/serialize.h file
		let mut out_script_length_bytes = [0; 1];
		let start_index = end_index;
		let end_index = start_index + 1;
		out_script_length_bytes.copy_from_slice(&result[start_index..end_index]);
		let out_script_length = get_decimal_value_usize(&out_script_length_bytes);

		output_size += 1;
		
		// println!("amount: {:?} {}", encode_hex(&amount_bytes), amount);
		// println!("out_script_length: {:?} {}", out_script_length_bytes, out_script_length);
		let mut address_bytes = vec![0; out_script_length];
		let start_index = end_index;
		let end_index = start_index + out_script_length;
		address_bytes.copy_from_slice(&result[start_index..end_index]);
		let script_pub_key = encode_hex(&address_bytes);

		index = end_index;
		output_size += end_index - start_index;

		let output = Output {
			amount,
			script_pub_key,
		};

		outputs.push(output);
	}

	// println!("vout: {:#?}", outputs);

	// List of witnesses
	if flag.is_some() {

	}

	// always 4 bytes long
	let mut lock_time_bytes = [0; 4];
	let start_index = index;
	let end_index = start_index + 4;
	lock_time_bytes.copy_from_slice(&result[start_index..end_index]);
	let lock_time= get_decimal_value_u32(&lock_time_bytes);


	let transaction = Transaction {
		version,
		flag,
		in_counter,
		inputs,
		out_counter,
		outputs,
		lock_time,
	};

	Ok(transaction)
}

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

pub fn encode_hex(bytes: &[u8]) -> String {
	let mut s = String::with_capacity(bytes.len() * 2);
	for &b in bytes {
		write(&mut s, format_args!("{:02x}", b)).unwrap();
	}
	s
}

// can't do i32::from_le_bytes because from_le_bytes requires a 4 byte input
// can convert 2 bytes to 4 bytes: https://dev.to/wayofthepie/three-bytes-to-an-integer-13g5
fn get_decimal_value_usize(bytes: &[u8; 1]) -> usize {
    ((bytes[0] as usize) <<  0) +
           ((0 as usize) <<  8)
}

fn get_decimal_value_u16(bytes: &[u8; 1]) -> u16 {
    ((bytes[0] as u16) <<  0) +
           ((0 as u16) <<  8)
}

fn get_decimal_value_u32(bytes: &[u8]) -> u32 {
	u32::from_le_bytes(bytes.try_into().unwrap()) 
}

fn get_decimal_value_i32(bytes: &[u8]) -> i32 {
	match i32::from_str_radix(&encode_hex(&bytes), 16) {
		Ok(val) => val,
		Err(e) => panic!("{}", e)
	}
}

fn get_decimal_value_i64(bytes: &[u8]) -> i64 {
	return i64::from_le_bytes(bytes.try_into().unwrap());
}

fn get_raw_transactions() -> Vec<String> {
	// txid: db6e06ff6e53356cc22cd1b9b8d951ddf70dc6bb275ee76880a0b951c1c290e6
	let data_pre_segwit= "{\"result\": \"02000000016dbad361f6a9f0c60e8b032e2008aa0a9151c7bf691464274c89315d2f6c52cc19000000fc0047304402204945c3e4f824d263bb22e117a12bfff741d996d594f07551c93e0fde77910d32022016c2b69daec51bd4afdd81bf90f76667dda515773b3da91174043fc7299acb5301473044022053c71a4730160b20e565cb669a44b793f42d2912e84d528cf203089abcb2874402203311303cfc36b91372e47d5fa0b22104e7c25bb5a8dcccd15c423620d5700304014c69522102047464f518269c6cba42b859d28e872ef8f6bb47d93e24d5c11ac6eca8a2845721029b48417598a2d2dab54ddddfca8e1a9c8d4967002180961f53a7748710c2176521036b1023b6c7ed689aaf3bc8ca9ee5c55da383ae0c44fc8b0fec91d6965dae5d5e53aeffffffff0450da1100000000001600141e129251311437eea493fce2a3644a5a1af8d40710731d00000000001976a9140ac4423b045a0c8ed5f4fb992256ed293a313ae088ac946b9b000000000017a914cd38af19a803de11ddcee3a45221ed9ac49140478761ea945a0000000017a9143572de0bb360f212ef8813a9e012f63a7035c9c98700000000\",\"error\": null,\"id\": null}".to_string();

	// txid: 68333a10b368e0d002098827fa3f348135fb728ade74d265e6abf41dfcb60a1c
	let data_segwit= "{\"result\": \"02000000000103d19441b832d4e24e4e10c08413b57c017785ea7407b373d4566e11ad94d8134c1c000000171600147c846a806f4d9e516c9fb2fe364f28eac4e3c3fcffffffff3d416a5941422eeecbcc0e3fe6aa7a88d00d22b67df149293e3c5bee10c4719a2c000000171600147c846a806f4d9e516c9fb2fe364f28eac4e3c3fcffffffff4f22589a292781a3cc2d636b9f1932f367305625a7874f8573b72b98ad73699600000000171600147c846a806f4d9e516c9fb2fe364f28eac4e3c3fcffffffff02f56468040000000017a9142c21151d54bd219dcc4c52e1cb38672dab8e36cc87c0276544000000001976a91439b1050dba04b1d1bc556c2dcdcb3874ba3dc11e88ac0247304402203ccede7995b26185574a050373cfe607f475f7d8ee6927647c496e3b45bf61a302202bd1ff88c7f4ee0b6f0c98f687dff9033f770b23985f590d178b9085df589101012103789a9d83798d4cbf688f9969a94084ee1655059e137b43492ee94dc4538790ab02483045022100b46ab18056655cc56b1778fd61a56f895c2f44c97f055ea0269d991efd181fb402206d651a5fb51081cfdb247a1d489b182f41e52434d7c4575bea30d2ce3d24087d012103789a9d83798d4cbf688f9969a94084ee1655059e137b43492ee94dc4538790ab02473044022069bf2ac34569565a62a1e0c12750104f494a906fefd2f2a462199c0d4bc235d902200c37ef333b453966cc5e84b178ec62125cbed83e0c0df4448c0fb331efa49e51012103789a9d83798d4cbf688f9969a94084ee1655059e137b43492ee94dc4538790ab00000000\",\"error\": null,\"id\": null}".to_string();

	let data_pre_segwit_two = "{\"result\": \"0100000001c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd3704000000004847304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901ffffffff0200ca9a3b00000000434104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac00286bee0000000043410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac00000000\",\"error\": null,\"id\": null}".to_string();

	return vec![data_segwit]
	// return vec![data_pre_segwit_two, data_segwit]
	// return vec![data_pre_segwit, data_pre_segwit_two, data_segwit]
}

