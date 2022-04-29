#![allow(dead_code)]

use std::error::Error;
use std::io::{BufRead, self};

use script::Script;
use transaction::Transaction;

mod txio;
mod opcodes;
mod transaction;
mod script;
mod hash;

pub trait Serialize {
	/// Create new type T from arguments provided by user
	fn new<R: BufRead>(reader: R) -> Self;
	fn as_hex(&self) -> String;
}

pub trait Deserialize {
	fn from_raw(data: String) -> Result<Self, Box<dyn Error>> where Self: Sized;
	fn as_asm(hex: String) -> String { "Not supported".to_string() } // Default implementation
}

fn main() {
	let transaction = decode_raw_transactions();
	println!("{:#?}", transaction);

	// let script = create_p2sh_scriptpubkey();
	// println!("script: {:02x?}", script);

	// let script = decode_script();
	// println!("{:#?}", script);
	
	// let transaction = Transaction::new(io::stdin().lock());
	// println!("{:#?}", transaction);
	// println!("{:#?}", transaction.as_hex());
}

fn create_p2sh_scriptpubkey() -> Script {
	// let script_hash = create_script_hash();
	Script::new(io::stdin().lock())
}

// fn create_script_hash() -> Vec<u8> {
//     // redeem script/original script
//     let bytes = b"this is a string";
//     serialize::create_hash160(bytes)
// }

fn decode_script() -> String {
	let script = "76a91414011f7254d96b819c76986c277d115efce6f7b58763ac67210394854aa6eab5b2a8122cc726e9dded053a2184d88256816826d6231c068d4a5b7c820120876475527c21030d417a46946384f88d5f3337267c5e579765875dc4daca813e21734b140639e752ae67a914b43e1b38138a41b37f7cd9a1d274bc63e3a9b5d188ac6868";

	Script::as_asm(script.to_string())
	// match Script::new(script) {
	//     Ok(s) => s,
	//     Err(e) => panic!("{}", e)
	// }
}

fn decode_raw_transactions() -> Transaction {
	// txid: db6e06ff6e53356cc22cd1b9b8d951ddf70dc6bb275ee76880a0b951c1c290e6
	let _data_pre_segwit= "{\"result\": \"02000000016dbad361f6a9f0c60e8b032e2008aa0a9151c7bf691464274c89315d2f6c52cc19000000fc0047304402204945c3e4f824d263bb22e117a12bfff741d996d594f07551c93e0fde77910d32022016c2b69daec51bd4afdd81bf90f76667dda515773b3da91174043fc7299acb5301473044022053c71a4730160b20e565cb669a44b793f42d2912e84d528cf203089abcb2874402203311303cfc36b91372e47d5fa0b22104e7c25bb5a8dcccd15c423620d5700304014c69522102047464f518269c6cba42b859d28e872ef8f6bb47d93e24d5c11ac6eca8a2845721029b48417598a2d2dab54ddddfca8e1a9c8d4967002180961f53a7748710c2176521036b1023b6c7ed689aaf3bc8ca9ee5c55da383ae0c44fc8b0fec91d6965dae5d5e53aeffffffff0450da1100000000001600141e129251311437eea493fce2a3644a5a1af8d40710731d00000000001976a9140ac4423b045a0c8ed5f4fb992256ed293a313ae088ac946b9b000000000017a914cd38af19a803de11ddcee3a45221ed9ac49140478761ea945a0000000017a9143572de0bb360f212ef8813a9e012f63a7035c9c98700000000\",\"error\": null,\"id\": null}".to_string();

	// txid: 68333a10b368e0d002098827fa3f348135fb728ade74d265e6abf41dfcb60a1c
	let _data_segwit= "{\"result\": \"02000000000103d19441b832d4e24e4e10c08413b57c017785ea7407b373d4566e11ad94d8134c1c000000171600147c846a806f4d9e516c9fb2fe364f28eac4e3c3fcffffffff3d416a5941422eeecbcc0e3fe6aa7a88d00d22b67df149293e3c5bee10c4719a2c000000171600147c846a806f4d9e516c9fb2fe364f28eac4e3c3fcffffffff4f22589a292781a3cc2d636b9f1932f367305625a7874f8573b72b98ad73699600000000171600147c846a806f4d9e516c9fb2fe364f28eac4e3c3fcffffffff02f56468040000000017a9142c21151d54bd219dcc4c52e1cb38672dab8e36cc87c0276544000000001976a91439b1050dba04b1d1bc556c2dcdcb3874ba3dc11e88ac0247304402203ccede7995b26185574a050373cfe607f475f7d8ee6927647c496e3b45bf61a302202bd1ff88c7f4ee0b6f0c98f687dff9033f770b23985f590d178b9085df589101012103789a9d83798d4cbf688f9969a94084ee1655059e137b43492ee94dc4538790ab02483045022100b46ab18056655cc56b1778fd61a56f895c2f44c97f055ea0269d991efd181fb402206d651a5fb51081cfdb247a1d489b182f41e52434d7c4575bea30d2ce3d24087d012103789a9d83798d4cbf688f9969a94084ee1655059e137b43492ee94dc4538790ab02473044022069bf2ac34569565a62a1e0c12750104f494a906fefd2f2a462199c0d4bc235d902200c37ef333b453966cc5e84b178ec62125cbed83e0c0df4448c0fb331efa49e51012103789a9d83798d4cbf688f9969a94084ee1655059e137b43492ee94dc4538790ab00000000\",\"error\": null,\"id\": null}".to_string();

	let _data_pre_segwit_two = "{\"result\": \"0100000001c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd3704000000004847304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901ffffffff0200ca9a3b00000000434104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac00286bee0000000043410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac00000000\",\"error\": null,\"id\": null}".to_string();

	let _data = "{\"result\": \"0100000001e11af7c4292505f99a4a5f4ff0818ac86c197bb16261f91af3f5cac661259c88000000006a473044022045c7199ffc8069a498135b7bb2678da16e8b5d49455b4a7ace755928c9339c7a022051cbf72024cf273444640f7b993b2bf3d329124b03e6744edaed5158a30e29b8012103fd9bc1e9803e739720e0f1c63e580a94656c7d0cab6cd083f0c0dfb221b90662ffffffff0200b080f6450100001976a9143b9552116adcc2fbd74fad44a4da603a727c816e88aca05ecf1c000100001976a914f90ce447f14847e841d4d2ecc76299b5bc77166188ac00000000\",\"error\": null,\"id\": null}".to_string();

	let raw_transaction = _data;
	// let raw_transaction = _data_segwit;
	// let raw_transaction = _data_pre_segwit_two;
	// let raw_transaction = _data_pre_segwit;

	match Transaction::from_raw(raw_transaction) {
		Ok(transaction) => transaction,
		Err(e) => panic!("{}", e)
	}
}
