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
	fn as_bytes(&self) -> &[u8];
}

pub trait Deserialize {
	fn decode_raw<R: BufRead>(reader: R) -> Result<Self, Box<dyn Error>> where Self: Sized;
	fn as_asm(&self) -> String { "Not supported".to_string() } // Default implementation
}

fn main() {
	println!("What would you like to do?");
	println!("1. Create  new transaction");
	println!("2. Create new script");
	println!("3. Decode raw transaction");
	println!("4. Decode raw script");

	let option = txio::user_read_u32(io::stdin().lock());

	if option == 1 {
		let transaction = Transaction::new(io::stdin().lock());
		println!();
		println!("{:#?}", transaction);
		println!("Raw transaction {:#?}", transaction.as_hex());
	} else if option == 2 {
		let script = Script::new(io::stdin().lock());
		println!();
		println!("ScriptPubKey asm: {:#?}", script.as_asm());
		println!("ScriptPubKey hex: {:#?}", script.as_hex());
	} else if option == 3 {
		let transaction = Transaction::decode_raw(io::stdin().lock());
		println!();
		println!("{:#?}", transaction);
	} else if option == 4 {
		// let script = decode_script();
		let script = Script::decode_raw(io::stdin().lock());
		println!();
		println!("{:#?}", script);
	} else {
		todo!()
	}
}
