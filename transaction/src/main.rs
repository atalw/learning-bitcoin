use std::error::Error;
use std::io::Cursor;

use serde_json::Value;

mod txio;

#[derive(Debug)]
#[allow(dead_code)]
struct Transaction {
	version: u32,
	flag: Option<u16>,
	in_counter: u64, // varint -> byte size 1-9
	inputs: Vec<Input>,
	out_counter: u64, // varint -> byte size 1-9
	outputs: Vec<Output>,
	lock_time: u32
}

#[derive(Debug)]
#[allow(dead_code)]
struct Input {
	previous_tx: String,
	tx_index: u32,
	script_sig: String,
	sequence: String,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Output {
	amount: u64,
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

	// convert to bytes
	let result: Vec<u8> = txio::decode_hex(d["result"].as_str().unwrap())?;
	let mut stream = Cursor::new(result.clone());

	// version: always 4 bytes long
	let version = txio::read_u32(&mut stream);

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

		let previous_tx = txio::read_hex256(&mut stream);
		let tx_index = txio::read_u32(&mut stream);
		// question: why are there n extra bytes in script_sig? in/out_script_length specifies it
		let in_script_length = txio::read_compact_size(&mut stream);
		let script_sig= txio::read_hex_var(&mut stream, in_script_length);
		let sequence = txio::read_hex32(&mut stream);

		let input = Input {
			previous_tx,
			tx_index,
			script_sig,
			sequence,
		};

		inputs.push(input);
	}

	// number of outputs
	let out_counter = txio::read_compact_size(&mut stream);
	assert_ne!(out_counter, 0);

	let mut outputs: Vec<Output> = Vec::new();
	for _ in 0..out_counter {

		let amount = txio::read_u64(&mut stream);
		let out_script_length = txio::read_compact_size(&mut stream);
		let script_pub_key = txio::read_hex_var(&mut stream, out_script_length);

		let output = Output {
			amount,
			script_pub_key,
		};

		outputs.push(output);
	}

	// List of witnesses
	if flag.is_some() {}

	// always 4 bytes long
	let lock_time = txio::read_u32(&mut stream);

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

fn get_raw_transactions() -> Vec<String> {
	// txid: db6e06ff6e53356cc22cd1b9b8d951ddf70dc6bb275ee76880a0b951c1c290e6
	let data_pre_segwit= "{\"result\": \"02000000016dbad361f6a9f0c60e8b032e2008aa0a9151c7bf691464274c89315d2f6c52cc19000000fc0047304402204945c3e4f824d263bb22e117a12bfff741d996d594f07551c93e0fde77910d32022016c2b69daec51bd4afdd81bf90f76667dda515773b3da91174043fc7299acb5301473044022053c71a4730160b20e565cb669a44b793f42d2912e84d528cf203089abcb2874402203311303cfc36b91372e47d5fa0b22104e7c25bb5a8dcccd15c423620d5700304014c69522102047464f518269c6cba42b859d28e872ef8f6bb47d93e24d5c11ac6eca8a2845721029b48417598a2d2dab54ddddfca8e1a9c8d4967002180961f53a7748710c2176521036b1023b6c7ed689aaf3bc8ca9ee5c55da383ae0c44fc8b0fec91d6965dae5d5e53aeffffffff0450da1100000000001600141e129251311437eea493fce2a3644a5a1af8d40710731d00000000001976a9140ac4423b045a0c8ed5f4fb992256ed293a313ae088ac946b9b000000000017a914cd38af19a803de11ddcee3a45221ed9ac49140478761ea945a0000000017a9143572de0bb360f212ef8813a9e012f63a7035c9c98700000000\",\"error\": null,\"id\": null}".to_string();

	// txid: 68333a10b368e0d002098827fa3f348135fb728ade74d265e6abf41dfcb60a1c
	let data_segwit= "{\"result\": \"02000000000103d19441b832d4e24e4e10c08413b57c017785ea7407b373d4566e11ad94d8134c1c000000171600147c846a806f4d9e516c9fb2fe364f28eac4e3c3fcffffffff3d416a5941422eeecbcc0e3fe6aa7a88d00d22b67df149293e3c5bee10c4719a2c000000171600147c846a806f4d9e516c9fb2fe364f28eac4e3c3fcffffffff4f22589a292781a3cc2d636b9f1932f367305625a7874f8573b72b98ad73699600000000171600147c846a806f4d9e516c9fb2fe364f28eac4e3c3fcffffffff02f56468040000000017a9142c21151d54bd219dcc4c52e1cb38672dab8e36cc87c0276544000000001976a91439b1050dba04b1d1bc556c2dcdcb3874ba3dc11e88ac0247304402203ccede7995b26185574a050373cfe607f475f7d8ee6927647c496e3b45bf61a302202bd1ff88c7f4ee0b6f0c98f687dff9033f770b23985f590d178b9085df589101012103789a9d83798d4cbf688f9969a94084ee1655059e137b43492ee94dc4538790ab02483045022100b46ab18056655cc56b1778fd61a56f895c2f44c97f055ea0269d991efd181fb402206d651a5fb51081cfdb247a1d489b182f41e52434d7c4575bea30d2ce3d24087d012103789a9d83798d4cbf688f9969a94084ee1655059e137b43492ee94dc4538790ab02473044022069bf2ac34569565a62a1e0c12750104f494a906fefd2f2a462199c0d4bc235d902200c37ef333b453966cc5e84b178ec62125cbed83e0c0df4448c0fb331efa49e51012103789a9d83798d4cbf688f9969a94084ee1655059e137b43492ee94dc4538790ab00000000\",\"error\": null,\"id\": null}".to_string();

	let data_pre_segwit_two = "{\"result\": \"0100000001c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd3704000000004847304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901ffffffff0200ca9a3b00000000434104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac00286bee0000000043410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac00000000\",\"error\": null,\"id\": null}".to_string();

	// return vec![data_segwit]
	return vec![data_pre_segwit_two, data_segwit]
	// return vec![data_pre_segwit, data_pre_segwit_two, data_segwit]
}

