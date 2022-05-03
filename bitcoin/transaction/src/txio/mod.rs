use std::fmt::{write, LowerHex};
use std::io::{Seek, SeekFrom, BufRead, Write, Error};
use std::num::ParseIntError;

use crate::Deserialize;
use crate::script::{ScriptBuilder, Script};

/// Discussion on Vec<u8> vs Box<[u8]>
/// https://github.com/ipld/libipld/issues/36
pub type HexBytes = Box<[u8]>;

/// All data in the Bitcoin network is encoded in a specific format. This is done so that nodes can
/// communicate with each other with a shared language. Encodable ensures that the data is encoded
/// in the correct format taking care of endianness.
pub trait Encodable {
	/// Convert an array of bytes, which is in little endian, into a Hex string.
	/// The most significant bit in little-endian is at the smallest memory address i.e. the
	/// number 123 is stored as 321.
	fn encode_hex_le(&self) -> String;
	/// Convert an array of bytes, which is in big endian, into a Hex string.
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

// TODO: Can't figure out how to wrap read into a loop so that the user can enter the text again
// incase of an error. The problem is when adding support for mock inputs using Cursor.
pub fn user_read_u32<R: BufRead>(reader: R) -> u32 {
	let mut line = String::new();
	match read_line(reader, &mut line) {
		Ok(n) => {
			if n <= 5 { // 4 bytes + \n
				match line.trim_end().parse::<u32>() {
					Ok(val) => return val,
					Err(e) => panic!("{} {}", e, line)
				}

			} else {
				panic!("Number is too big");
			}
		},
		Err(e) => panic!("{}", e)
	}
}

pub fn user_read_u64<R: BufRead>(reader: R) -> u64 {
	let mut line = String::new();
	match read_line(reader, &mut line) {
		Ok(n) => {
			if n <= 9 { // 8 bytes + \n
				match line.trim_end().parse::<u64>() {
					Ok(val) => return val,
					Err(e) => panic!("{} {}", e, line)
				}

			} else {
				panic!("Number is too big");
			}
		},
		Err(e) => panic!("{}. Try again!", e)
	}
}

pub fn user_read_bool<R: BufRead>(reader: R) -> bool {
	let mut line = String::new();
	match read_line(reader, &mut line) {
		Ok(_) => {
			match line.trim_end().parse::<bool>() {
				Ok(val) => return val,
				Err(e) => panic!("{}", e)
			}
		},
		Err(e) => panic!("{}", e)
	}
}

pub fn user_read_hex<R: BufRead>(reader: R, len: Option<u64>) -> String {
	let mut line = String::new();
	match read_line(reader, &mut line) {
		Ok(n) => {
			if let Some(b) = len { 
				if (n as u64 - 1) / 2 == b {
					return line.trim_end().to_string()
				} else {
					panic!("Expected {} bytes, got {} bytes", b, n-1);
				}
			} else {
				return line.trim_end().to_string()
			}
		},
		Err(e) => panic!("{}", e)
	}
}

/// Used for reach script_pub_key and script_sig
pub fn user_read_script_hex<R: BufRead>(reader: R) -> Script {
	let mut line = String::new();
	match read_line(reader, &mut line) {
		Ok(_) => {
			Script(line
				   .trim_end()
				   .decode_hex_be()
				   .expect("Is the script_pub_key/script_sig correct?")
			)
		},
		Err(e) => panic!("{}", e)
	}
}

pub fn user_read_script_asm<R: BufRead>(reader: R) -> Script {
	let mut line = String::new();
	match read_line(reader, &mut line) {
		Ok(_) => {
			parse_asm_script(line.trim_end().to_string())
		},
		Err(e) => panic!("{}", e)
	}
}

fn parse_asm_script(script_asm: String) -> Script {
	let tokens: Vec<&str> = script_asm.split(" ").collect();

	let mut script_builder = ScriptBuilder::new();
	for token in &tokens {
		script_builder.push(token);
	}
	let script = script_builder.into_script();
	println!("Parsed script is: {}", script.as_asm());
	script
}

fn read_line<R>(mut reader: R, line: &mut String) -> Result<usize, Error>
where
    R: BufRead,
{
    reader.read_line(line)
}


// sometimes can't do i32::from_le_bytes because from_le_bytes requires a 4 byte input
// can convert 2 bytes to 4 bytes: https://dev.to/wayofthepie/three-bytes-to-an-integer-13g5
