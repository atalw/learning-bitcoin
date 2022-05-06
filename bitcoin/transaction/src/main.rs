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
mod key;

/// Bitcoin transactions follow a specific encoding so that each node in the network can
/// communicate in a shared language. Serialize is a trait which different components of a
/// transaction implement to convert human-readable data into Bitcoin-readable data.
pub trait Serialize {
	/// Interactively create a Bitcoin transaction component by accepting user input for all fields. 
	/// Helpful user prompts are shown to help the user enter the data in the correct format.
	/// Returns a transaction component type like Script or a whole Transaction.
	fn encode_raw<R: BufRead>(reader: R) -> Self;
	/// Convert a transaction component type to a Hex string.
	fn as_hex(&self) -> String;
}

/// Convert Bitcoin consensus encoded data into transaction components.
pub trait Deserialize {
	/// Given Hex bytes, convert it into transaction components which allow it to be
	/// human-readable. This is similar to `decoderawtransaction` and `decodescript` found in the
	/// bitcoin-cli.
	fn decode_raw(bytes: HexBytes) -> Result<Self, Box<dyn Error>> where Self: Sized;
	/// Convert a transaction component to bytes.
	fn as_bytes(&self) -> HexBytes;
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
		let script = ScriptPubKey::decode_raw(hexbytes);
		println!();
		println!("{:#?}", script);
	} else {
		todo!()
	}
}
