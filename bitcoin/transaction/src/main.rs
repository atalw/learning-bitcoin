use std::error::Error;
use std::io::{BufRead, self};
use transaction::Transaction;
use txio::HexBytes;
use crate::script::ScriptPubKey;
use crate::txio::UserReadExt;

mod txio;
mod opcodes;
mod transaction;
mod script;
mod hash;

/// 
pub trait Serialize {
	fn encode_raw<R: BufRead>(reader: R) -> Self;
	fn as_hex(&self) -> String;
	fn as_bytes(&self) -> HexBytes;
}

pub trait Deserialize {
	fn decode_raw(bytes: HexBytes) -> Result<Self, Box<dyn Error>> where Self: Sized;
	// fn as_asm(&self) -> String { unimplemented!() }
}

fn main() {
	println!("What would you like to do?");
	println!("1. Create  new transaction");
	println!("2. Create new script");
	println!("3. Decode raw transaction");
	println!("4. Decode raw script");

	let option = io::stdin().lock().user_read_u32();

	if option == 1 {
		let transaction = Transaction::encode_raw(io::stdin().lock());
		println!();
		println!("{:#?}", transaction);
		println!("Raw transaction {:#?}", transaction.as_hex());
	} else if option == 2 {
		let script = ScriptPubKey::encode_raw(io::stdin().lock());
		println!();
		println!("script_pub_key: {:#?}", script);
	} else if option == 3 {
		println!("Enter a raw transaction hex");
		let hexbytes = io::stdin().lock().user_read_hex_var();
		let transaction = Transaction::decode_raw(hexbytes);
		println!();
		println!("{:#?}", transaction);
	} else if option == 4 {
		println!("Enter a raw script hex");
		let hexbytes = io::stdin().lock().user_read_hex_var();
		println!("{:?}", hexbytes);
		let script = ScriptPubKey::decode_raw(hexbytes);
		println!();
		println!("{:#?}", script);
	} else {
		todo!()
	}
}
