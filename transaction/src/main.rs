use std::error::Error;
use std::fmt::write;
use std::num::ParseIntError;

use serde_json::Value;

#[derive(Debug)]
struct Input {
	txid: String,
	tx_index: String,
	script_sig: String,
	sequence: String,
}

#[derive(Debug)]
struct Output {
	value: i64,
	address: String,
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

fn main() {
	let data_pre_segwit= "{\"result\": \"02000000016dbad361f6a9f0c60e8b032e2008aa0a9151c7bf691464274c89315d2f6c52cc19000000fc0047304402204945c3e4f824d263bb22e117a12bfff741d996d594f07551c93e0fde77910d32022016c2b69daec51bd4afdd81bf90f76667dda515773b3da91174043fc7299acb5301473044022053c71a4730160b20e565cb669a44b793f42d2912e84d528cf203089abcb2874402203311303cfc36b91372e47d5fa0b22104e7c25bb5a8dcccd15c423620d5700304014c69522102047464f518269c6cba42b859d28e872ef8f6bb47d93e24d5c11ac6eca8a2845721029b48417598a2d2dab54ddddfca8e1a9c8d4967002180961f53a7748710c2176521036b1023b6c7ed689aaf3bc8ca9ee5c55da383ae0c44fc8b0fec91d6965dae5d5e53aeffffffff0450da1100000000001600141e129251311437eea493fce2a3644a5a1af8d40710731d00000000001976a9140ac4423b045a0c8ed5f4fb992256ed293a313ae088ac946b9b000000000017a914cd38af19a803de11ddcee3a45221ed9ac49140478761ea945a0000000017a9143572de0bb360f212ef8813a9e012f63a7035c9c98700000000\",\"error\": null,\"id\": null}".to_string();

	// txid: 68333a10b368e0d002098827fa3f348135fb728ade74d265e6abf41dfcb60a1c
	let data_segwit= "{\"result\": \"02000000000103d19441b832d4e24e4e10c08413b57c017785ea7407b373d4566e11ad94d8134c1c000000171600147c846a806f4d9e516c9fb2fe364f28eac4e3c3fcffffffff3d416a5941422eeecbcc0e3fe6aa7a88d00d22b67df149293e3c5bee10c4719a2c000000171600147c846a806f4d9e516c9fb2fe364f28eac4e3c3fcffffffff4f22589a292781a3cc2d636b9f1932f367305625a7874f8573b72b98ad73699600000000171600147c846a806f4d9e516c9fb2fe364f28eac4e3c3fcffffffff02f56468040000000017a9142c21151d54bd219dcc4c52e1cb38672dab8e36cc87c0276544000000001976a91439b1050dba04b1d1bc556c2dcdcb3874ba3dc11e88ac0247304402203ccede7995b26185574a050373cfe607f475f7d8ee6927647c496e3b45bf61a302202bd1ff88c7f4ee0b6f0c98f687dff9033f770b23985f590d178b9085df589101012103789a9d83798d4cbf688f9969a94084ee1655059e137b43492ee94dc4538790ab02483045022100b46ab18056655cc56b1778fd61a56f895c2f44c97f055ea0269d991efd181fb402206d651a5fb51081cfdb247a1d489b182f41e52434d7c4575bea30d2ce3d24087d012103789a9d83798d4cbf688f9969a94084ee1655059e137b43492ee94dc4538790ab02473044022069bf2ac34569565a62a1e0c12750104f494a906fefd2f2a462199c0d4bc235d902200c37ef333b453966cc5e84b178ec62125cbed83e0c0df4448c0fb331efa49e51012103789a9d83798d4cbf688f9969a94084ee1655059e137b43492ee94dc4538790ab00000000\",\"error\": null,\"id\": null}".to_string();

	// match parse_raw_data(data_pre_segwit) {
	//     Ok(()) => {},
	//     Err(e) => panic!("{}", e)
	// }

	match parse_raw_data(data_segwit) {
		Ok(()) => {},
		Err(e) => panic!("{}", e)
	}
}

// can't do i32::from_le_bytes because from_le_bytes requires a 4 byte input
// can convert 2 bytes to 4 bytes: https://dev.to/wayofthepie/three-bytes-to-an-integer-13g5
fn get_decimal_value(bytes: &[u8]) -> i32 {
	match i32::from_str_radix(&encode_hex(&bytes), 16) {
		Ok(val) => val,
		Err(e) => panic!("{}", e)
	}
}

fn get_decimal_value_i64(bytes: &[u8]) -> i64 {
	return i64::from_le_bytes(bytes.try_into().unwrap());
}


fn parse_raw_data(data: String) -> Result<(), Box<dyn Error>> {
	
	let d: Value = serde_json::from_str(&data)?;
	println!("input hex: {}", d["result"]);
	println!("-------------------");

	let result: Vec<u8> = decode_hex(d["result"].as_str().unwrap())?;

	println!("input bytes: {:?}", result);
	println!("-------------------");

	let mut version = [0; 4];
	version.copy_from_slice(&result[..4]);

	println!("version: {:x?}", i32::from_le_bytes(version));

	let mut segwit_bytes = [0; 2];
	segwit_bytes.copy_from_slice(&result[4..6]);

	println!("segwit bytes: {:?}", encode_hex(&segwit_bytes));

	let mut number_of_inputs_bytes = [0; 1];
	number_of_inputs_bytes.copy_from_slice(&result[6..7]);
	let number_of_inputs = get_decimal_value(&number_of_inputs_bytes);
	assert_ne!(number_of_inputs, 0);
	println!("number of inputs: {:?}", number_of_inputs);

	let mut inputs: Vec<Input> = Vec::new();
	let mut index: usize = 0;
	for i in 0..number_of_inputs {

		// start_index: 7
		// start_index: 7 + 64 (each input is 64 bytes)
		let mut txid_bytes = [0; 32];
		let start_index = (7 + (i*64)) as usize;
		let end_index = start_index + 32;
		txid_bytes.copy_from_slice(&result[start_index..end_index]);
		let txid = encode_hex(&txid_bytes);

		let mut tx_index_bytes = [0; 4];
		let start_index = end_index;
		let end_index = start_index + 4;
		tx_index_bytes.copy_from_slice(&result[start_index..end_index]);
		let tx_index = encode_hex(&tx_index_bytes);

		let mut script_sig_bytes = [0; 24];
		let start_index = end_index;
		let end_index = start_index + 24;
		script_sig_bytes.copy_from_slice(&result[start_index..end_index]);
		let script_sig= encode_hex(&script_sig_bytes);

		let mut sequence_bytes = [0; 4];
		let start_index = end_index;
		let end_index = start_index + 4;
		index = end_index;
		sequence_bytes.copy_from_slice(&result[start_index..end_index]);
		let sequence = encode_hex(&sequence_bytes);

		let input = Input {
			txid,
			tx_index,
			script_sig,
			sequence,
		};

		inputs.push(input);
	}

	println!("vin: {:#?}", inputs);

	let mut number_of_outputs_bytes = [0; 1];
	number_of_outputs_bytes.copy_from_slice(&result[index..index+1]);
	let number_of_outputs = get_decimal_value(&number_of_outputs_bytes);
	assert_ne!(number_of_outputs, 0);
	println!("number of outputs: {:?}", number_of_outputs);

	let mut outputs: Vec<Output> = Vec::new();
	let index = index + 1;

	for i in 0..number_of_outputs {

		// start_index: index + 32 (each output is 32 bytes)
		let mut value_bytes= [0; 8];
		let start_index = index + (i*32) as usize;
		let end_index = start_index + 8;
		value_bytes.copy_from_slice(&result[start_index..end_index]);
		let value = get_decimal_value_i64(&value_bytes);

		let mut address_bytes = [0; 24];
		let start_index = end_index;
		let end_index = start_index + 24;
		address_bytes.copy_from_slice(&result[start_index..end_index]);
		let address = encode_hex(&address_bytes);

		let output = Output {
			value,
			address,
		};

		// TODO: question:   why are there 2 extra bytes?
		// output address 1: a9142c21151d54bd219dcc4c52e1cb38672dab8e36cc87
		// output address 2: 76a91439b1050dba04b1d1bc556c2dcdcb3874ba3dc11e88ac
		outputs.push(output);
	}

	println!("vout: {:#?}", outputs);

	Ok(())

}
