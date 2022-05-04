use std::fmt::{write, LowerHex};
use std::io::{Seek, SeekFrom, BufRead, Write, Error, ErrorKind};
use std::num::ParseIntError;
use crate::script::{ScriptBuilder, Script, ScriptAsm, ScriptPubKey};

// TODO:
// - Custom errors

/// Discussion on Vec<u8> vs Box<[u8]>
/// https://github.com/ipld/libipld/issues/36
pub type HexBytes = Box<[u8]>;

/// All data in the Bitcoin network is encoded in a specific format. This is done so that nodes can
/// communicate with each other with a shared language. Encodable ensures that the data is encoded
/// in the correct format taking care of endianness.
pub trait Encodable {
	/// Convert an array of bytes, which are in little endian, into a Hex string.
	/// The most significant bit in little-endian is at the smallest memory address i.e. the
	/// number 123 is stored as 321.
	fn encode_hex_le(&self) -> String;
	/// Convert an array of bytes, which are in big endian, into a Hex string.
	fn encode_hex_be(&self) -> String;
}

/// Operations on data in the Bitcoin network are done at the byte level. For example, if a sha256
/// needs to be calculated for a script, the hashing done on the hex string vs it done on the bytes
/// produces different results. Decodable converts a string into Hex bytes taking care of
/// endianness.
pub trait Decodable {
	/// Read a hex string into bytes in little-endian.
	fn decode_hex_le(&self) -> Result<HexBytes, ParseIntError>;
	/// Read a hex string into bytes in big-endian.
	fn decode_hex_be(&self) -> Result<HexBytes, ParseIntError>;
}

// TODO: Can this be broken?
impl<T: Sized + LowerHex> Encodable for [T] {
	fn encode_hex_le(&self) -> String {
		let mut s = String::with_capacity(self.len() * 2);
		for b in self.iter().rev() {
			write(&mut s, format_args!("{:02x}", b)).unwrap();
		}
		s
	}

	fn encode_hex_be(&self) -> String {
		let mut s = String::with_capacity(self.len() * 2);
		for b in self {
			write(&mut s, format_args!("{:02x}", b)).unwrap();
		}
		s
	}
}

impl<T: Sized + ToString> Decodable for T {
	fn decode_hex_le(&self) -> Result<HexBytes, ParseIntError> {
		let s =  &self.to_string();
		(0..s.len())
			.step_by(2)
			.rev()
			.map(|i| u8::from_str_radix(&s[i..i + 2], 16))
			.collect()
	}

	/// Read a hex string into bytes
	fn decode_hex_be(&self) -> Result<HexBytes, ParseIntError> {
		let s =  &self.to_string();
		(0..s.len())
			.step_by(2)
			.map(|i| u8::from_str_radix(&s[i..i + 2], 16))
			.collect()
	}
}

/// Extension of Read to decode data according to Bitcoin's spec.
pub trait ReadExt {
	/// Read 8-bits in where the source is in little-endian format.
	fn read_u8_le(&mut self) -> Result<u8, Error>;
	/// Read 16-bits in where the source is in little-endian format.
	fn read_u16_le(&mut self) -> Result<u16, Error>;
	/// Read 32-bits in where the source is in little-endian format.
	fn read_u32_le(&mut self) -> Result<u32, Error>;
	/// Read 32-bits in where the source is in little-endian format.
	fn read_u64_le(&mut self) -> Result<u64, Error>;

	/// Read 8-bits in where the source is in big-endian format.
	fn read_u8_be(&mut self) -> Result<u8, Error>;
	/// Read 16-bits in where the source is in big-endian format.
	fn read_u16_be(&mut self) -> Result<u16, Error>;
	/// Read 32-bits in where the source is in big-endian format.
	fn read_u32_be(&mut self) -> Result<u32, Error>;
	/// Read 64-bits in where the source is in big-endian format.
	fn read_u64_be(&mut self) -> Result<u64, Error>;

	/// Read 32-bits as bytes (which are in hex format).
	fn read_hex32(&mut self) -> Result<HexBytes, Error>;
	/// Read 256-bits as bytes (which are in hex format).
	fn read_hex256(&mut self) -> Result<HexBytes, Error>;
	/// Read an arbitrary number of bits as bytes (which are in hex format).
	fn read_hex_var(&mut self, len: u64) -> Result<HexBytes, Error>;

	/**
	 *
	 * Compact Size
	 * https://en.bitcoin.it/wiki/Protocol_documentation#Variable_length_integer
	 * size <  253        -- 1 byte
	 * size <= USHRT_MAX  -- 3 bytes  (253 + 2 bytes)
	 * size <= UINT_MAX   -- 5 bytes  (254 + 4 bytes)
	 * size >  UINT_MAX   -- 9 bytes  (255 + 8 bytes)
	 * fc -> 0-252
	 * fd -> 0000 (253 + 2 bytes)
	 * fe -> 0000 0000 (254 + 4 bytes)
	 * ff -> 0000 0000 0000 0000 (255 + 8 bytes)
	 * check bitcoin/src/serialize.h file
	 */
	fn read_compact_size(&mut self) -> Result<u64, Error>;

	/// Seek the current buffer forward or backward by a specified amount.
	fn seek_from_curr(&mut self, length: i64);
}

/// Extension of Write to decode data according to Bitcoin's spec.
pub trait WriteExt {
	/// Write 16-bits on to the buffer in little-endian format. Return the number of bytes written.
	fn write_u16_le(&mut self, val: u16) -> usize;
	/// Write 32-bits on to the buffer in little-endian format. Return the number of bytes written.
	fn write_u32_le(&mut self, val: u32) -> usize;
	/// Write 64-bits on to the buffer in little-endian format. Return the number of bytes written.
	fn write_u64_le(&mut self, val: u64) -> usize;

	/// Write 16-bits on to the buffer in big-endian format. Return the number of bytes written.
	fn write_u16_be(&mut self, val: u16) -> usize;

	/// Write an array of hex bytes on to the buffer (in big-endian). Specify if the len as
	/// compact-size should be written first. This is useful for scripts.
	/// Returns the number of bytes written.
	fn write_hex(&mut self, val: HexBytes, with_varint: bool) -> usize;

	/// Given a size, write it to the buffer in Bitcoin varint/compact-size format.
	fn write_varint(&mut self, size: u64) -> usize;
}

// ---- Buffer reading ----
macro_rules! impl_read_int_le {
	($ty: ty, $len: expr, $fn_name: ident) => {
		fn $fn_name(&mut self) -> Result<$ty, Error> {
			let mut bytes = [0; $len];
			self.read(&mut bytes)?;
			Ok(<$ty>::from_le_bytes(bytes))
		}
	};
}

macro_rules! impl_read_int_be {
	($ty: ty, $len: expr, $fn_name: ident) => {
		fn $fn_name(&mut self) -> Result<$ty, Error> {
			let mut bytes = [0; $len];
			self.read(&mut bytes)?;
			Ok(<$ty>::from_be_bytes(bytes))
		}
	};
}

macro_rules! impl_read_hex {
	($len: expr, $fn_name: ident) => {
		fn $fn_name(&mut self) -> Result<HexBytes, Error> {
			let mut bytes = [0; $len];
			self.read(&mut bytes)?;
			Ok(Box::new(bytes))
		}
	};
}

impl<R: BufRead + Seek> ReadExt for R {
	impl_read_int_le!(u8, 1, read_u8_le);
	impl_read_int_le!(u16, 2, read_u16_le);
	impl_read_int_le!(u32, 4, read_u32_le);
	impl_read_int_le!(u64, 8, read_u64_le);

	impl_read_int_be!(u8, 1, read_u8_be);
	impl_read_int_be!(u16, 2, read_u16_be);
	impl_read_int_be!(u32, 4, read_u32_be);
	impl_read_int_be!(u64, 8, read_u64_be);

	impl_read_hex!(4, read_hex32);
	impl_read_hex!(32, read_hex256);

	fn read_hex_var(&mut self, length: u64) -> Result<HexBytes, Error> {
		let mut bytes = vec![0; length as usize];
		self.read(&mut bytes)?;
		// There has to be a better way...
		Ok(bytes.encode_hex_be().decode_hex_be().expect("unable to parse"))
	}

	fn read_compact_size(&mut self) -> Result<u64, Error> {
		let  varint_size: u8 = self.read_u8_le()?;
		let size: u64;

		if varint_size < 253 {
			size = varint_size as u64;
		} else if varint_size == 253 {
			size = self.read_u16_le()? as u64;
			assert!(size > 253);
		} else if varint_size == 254 {
			size = self.read_u32_le()? as u64;
			assert!(size > 0x10000);
		} else if varint_size == 255 {
			size = self.read_u64_le()?;
			assert!(size > 0x100000000);
		} else {
			// TODO: Should i return an error over here? How?
			panic!()
		}

		assert!(size != 0);
		Ok(size)
	}

	fn seek_from_curr(&mut self, length: i64) {
		match self.seek(SeekFrom::Current(length)) {
			Ok(_) => (),
			Err(e) => panic!("{}", e)
		}
	}
}

macro_rules! impl_write_int_le {
	($ty: ty, $fn_name: ident) => {
		fn $fn_name(&mut self, val: $ty) -> usize {
			let bytes = val.to_le_bytes();
			match self.write(&bytes) {
				Ok(n) => n,
				Err(e) => panic!("{}", e)
			}
		}
	};
}

macro_rules! impl_write_int_be {
	($ty: ty, $fn_name: ident) => {
		fn $fn_name(&mut self, val: $ty) -> usize {
			let bytes = val.to_be_bytes();
			match self.write(&bytes) {
				Ok(n) => n,
				Err(e) => panic!("{}", e)
			}
		}
	};
}


impl<W: Write> WriteExt for W {
	impl_write_int_le!(u16, write_u16_le);
	impl_write_int_le!(u32, write_u32_le);
	impl_write_int_le!(u64, write_u64_le);

	impl_write_int_be!(u16, write_u16_be);

	fn write_hex(&mut self, bytes: HexBytes, with_varint: bool) -> usize {
		if with_varint { self.write_varint(bytes.len() as u64); }
		match self.write(&bytes) {
			Ok(n) => n,
			Err(e) => panic!("{}", e)
		}
	}

	fn write_varint(&mut self, size: u64) -> usize {
		let mut bytes: Vec<u8> = Vec::new();
		if size < 253 {
			bytes.push(size as u8);
		} else if size < 0x100 {
			bytes.push(253);
			bytes.push(size as u8);
		} else if size < 0x10000 {
			bytes.push(254);
			bytes.push((size % 0x100) as u8);
			bytes.push((size / 0x100) as u8);
		} else if size < 0x100000000 {
			bytes.push(255);
			bytes.push((size % 0x100) as u8);
			bytes.push(((size / 0x100) % 0x100) as u8);
			bytes.push(((size % 0x10000) % 0x100) as u8);
			bytes.push((size / 0x1000000) as u8);
		} else {
			panic!()
		}

		match self.write(&bytes) {
			Ok(n) => n,
			Err(e) => panic!("{}", e)
		}
	}
}

/// Extensions on BufRead to accept user input. BufRead so that mock inputs can be simulated for
/// testing purposes. This is different from ReadExt in that each function reads a line as an input
/// and repeats until a valid input is parsed.
pub trait UserReadExt {
	/// Read a 32-bit unsigned integer.
	fn user_read_u32(&mut self) -> u32;
	/// Read a 64-bit unsigned integer.
	fn user_read_u64(&mut self) -> u64;

	/// Read a 32-bit (4 byte) hex string and return a bytes array.
	fn user_read_hex32(&mut self) -> HexBytes;
	/// Read a 256-bit (32 byte) hex string and return a bytes array.
	fn user_read_hex256(&mut self) -> HexBytes;
	/// Read a variable length hex string and return a bytes array.
	fn user_read_hex_var(&mut self) -> HexBytes;

	/// Read a "true" or "false" input and return a bool.
	fn user_read_bool(&mut self) -> bool;

	/// Read an assembly script with opcodes until a valid script is parsed.
	/// Note: The script needs to be verbose in that if the script contains a number 1, OP_1 needs
	/// to be entered otherwise it won't parse correctly.
	fn user_read_asm(&mut self) -> HexBytes;
}

macro_rules! user_read_int {
	($ty: ty, $fn_name: ident) => {
		fn $fn_name(&mut self) -> $ty {
			loop {
				let mut line = String::new();
				match self.read_line(&mut line) {
					Ok(n) if n <= (<$ty>::BITS/8 + 1) as usize => {
						match line.trim_end().parse::<$ty>() {
							Ok(v) => return v,
							Err(_) => println!("Error! Enter a number.")
						}
					},
					_ => println!("Try again")
				}
			}
		}

	};
}

macro_rules! user_read_hex {
	($len: expr, $fn_name: ident) => {
		fn $fn_name(&mut self) -> HexBytes {
			loop {
				let mut line = String::new();
				match self.read_line(&mut line) {
					Ok(n) if n == $len*2 + 1 => {
						match line.trim_end().decode_hex_be() {
							Ok(hex) => return hex,
							Err(e) => println!("{}. Try again.", e)
						}
					},
					_ => panic!("Error! Try again.")
				}
			}
		}
	};
}

impl<R: BufRead> UserReadExt for R {
	user_read_int!(u32, user_read_u32);
	user_read_int!(u64, user_read_u64);

	user_read_hex!(4, user_read_hex32);
	user_read_hex!(32, user_read_hex256);

	fn user_read_hex_var(&mut self) -> HexBytes {
		loop {
			let mut line = String::new();
			match self.read_line(&mut line) {
				Ok(_) => {
					match line.trim_end().decode_hex_be() {
						Ok(hex) => return hex,
						Err(e) => println!("{}. Try again.", e)
					}
				},
				Err(e) => println!("{}! Try again.", e)
			}
		}
	}

	fn user_read_bool(&mut self) -> bool {
		loop {
			let mut line = String::new();
			match self.read_line(&mut line) {
				Ok(_) => {
					match line.trim_end().parse::<bool>() {
						Ok(v) => return v,
						Err(_) => println!("Error! Enter \"true\" or \"false\".")
					}
				},
				_ => println!("Try again")
			}
		}
	}

	fn user_read_asm(&mut self) -> HexBytes {
		loop {
			let mut line = String::new();
			match self.read_line(&mut line) {
				Ok(_) => {
					match line.trim_end().parse_asm() {
						Ok(bytes) => return bytes,
						Err(e) => println!("{}. Try again.", e)
					}

				},
				Err(e) => println!("{}. Try again.", e)
			}
		}
	}
}

trait StrExt {
	// FIXME: return type
	fn parse_asm(&self) -> Result<HexBytes, Box<dyn std::error::Error>>;
}

impl StrExt for str {
	fn parse_asm(&self) -> Result<HexBytes, Box<dyn std::error::Error>> {
		let tokens: Vec<&str> = self.split(" ").collect();

		let mut script_builder = ScriptBuilder::new();
		for token in &tokens {
			println!("{:?}", token);
			script_builder.push(token)?;
		}
		let script: ScriptPubKey = script_builder.into_script();
		println!("Parsed script is: {}", script.as_asm());
		if script.as_asm() == self {
			Ok(script.script)
		} else {
			Err(Box::new(Error::new(ErrorKind::InvalidInput, "Uh oh! The parsed script does not match.")))
		}
	}
}
