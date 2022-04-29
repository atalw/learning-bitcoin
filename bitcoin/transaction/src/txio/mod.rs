use std::fmt::write;
use std::io::{Read, Cursor, Seek, SeekFrom, BufRead, Write, Error};
use std::num::ParseIntError;

// ---- Conversions ----

pub fn decode_hex_le(s: &str) -> Result<Vec<u8>, ParseIntError> {
	(0..s.len())
		.step_by(2)
		.rev()
		.map(|i| u8::from_str_radix(&s[i..i + 2], 16))
		.collect()
}

pub fn decode_hex_be(s: &str) -> Result<Vec<u8>, ParseIntError> {
	(0..s.len())
		.step_by(2)
		.map(|i| u8::from_str_radix(&s[i..i + 2], 16))
		.collect()
}

pub fn encode_hex_le(bytes: &[u8]) -> String {
	let mut s = String::with_capacity(bytes.len() * 2);
	for &b in bytes.iter().rev() {
		write(&mut s, format_args!("{:02x}", b)).unwrap();
	}
	s
}

pub fn encode_hex_be(bytes: &[u8]) -> String {
	let mut s = String::with_capacity(bytes.len() * 2);
	for &b in bytes {
		write(&mut s, format_args!("{:02x}", b)).unwrap();
	}
	s
}

// ---- Buffer reading ----

/**
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
pub fn read_compact_size(stream: &mut Cursor<Vec<u8>>) -> u64 {
	let  varint_size: u8 = read_u8_le(stream);
	let size: u64;

	if varint_size < 253 {
		size = varint_size as u64;
	} else if varint_size == 253 {
		size = read_u16_le(stream) as u64;
		assert!(size > 253);
	} else if varint_size == 254 {
		size = read_u32_le(stream) as u64;
		assert!(size > 0x10000);
	} else if varint_size == 255 {
		size = read_u64_le(stream);
		assert!(size > 0x100000000);
	} else {
		panic!()
	}

	assert!(size != 0);
	size
}

pub fn read_u8_le(stream: &mut Cursor<Vec<u8>>) -> u8 {
	let mut bytes = [0; 1];
	match stream.read(&mut bytes) {
		Ok(_) => u8::from_le_bytes(bytes),
		Err(e) => panic!("{}", e)
	}
}

pub fn read_u16_le(stream: &mut Cursor<Vec<u8>>) -> u16 {
	let mut bytes = [0; 2];
	match stream.read(&mut bytes) {
		Ok(_) => u16::from_le_bytes(bytes),
		Err(e) => panic!("{}", e)
	}
}
pub fn read_u16_be(stream: &mut Cursor<Vec<u8>>) -> u16 {
	let mut bytes = [0; 2];
	match stream.read(&mut bytes) {
		Ok(_) => u16::from_be_bytes(bytes),
		Err(e) => panic!("{}", e)
	}
}

pub fn read_u32_le(stream: &mut Cursor<Vec<u8>>) -> u32 {
	let mut bytes = [0; 4];
	match stream.read(&mut bytes) {
		Ok(_) => u32::from_le_bytes(bytes),
		Err(e) => panic!("{}", e)
	}
}

pub fn read_u64_le(stream: &mut Cursor<Vec<u8>>) -> u64 {
	let mut bytes = [0; 8];
	match stream.read(&mut bytes) {
		Ok(_) => u64::from_le_bytes(bytes),
		Err(e) => panic!("{}", e)
	}
}

pub fn read_hex32_le(stream: &mut Cursor<Vec<u8>>) -> String {
	let mut bytes = [0; 4];
	match stream.read(&mut bytes) {
		Ok(_) => encode_hex_le(&bytes),
		Err(e) => panic!("{}", e)
	}
}

pub fn read_hex256_le(stream: &mut Cursor<Vec<u8>>) -> String {
	let mut bytes = [0; 32];
	match stream.read(&mut bytes) {
		Ok(_) => encode_hex_le(&bytes),
		Err(e) => panic!("{}", e)
	}
}

pub fn read_hex_var_be(stream: &mut Cursor<Vec<u8>>, length: u64) -> String {
	let mut bytes = vec![0; length as usize];
	match stream.read(&mut bytes) {
		Ok(_) => encode_hex_be(&bytes),
		Err(e) => panic!("{}", e)
	}
}

pub fn unread(stream: &mut Cursor<Vec<u8>>, length: i64) {
	match stream.seek(SeekFrom::Current(length)) {
		Ok(_) => (),
		Err(e) => panic!("{}", e)
	}
}

// TODO: Can't figure out how to wrap read into a loop so that the user can enter the text again
// and using Cursor for mock inputs.
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

fn read_line<R>(mut reader: R, line: &mut String) -> Result<usize, Error>
where
    R: BufRead,
{
    reader.read_line(line)
}

pub fn write_u16_le(stream: &mut Cursor<Vec<u8>>, val: u16) {
	let bytes = val.to_le_bytes();
	match stream.write(&bytes) {
		Ok(_) => {},
		Err(e) => panic!("{}", e)
	}
}

pub fn write_u16_be(stream: &mut Cursor<Vec<u8>>, val: u16) {
	let bytes = val.to_be_bytes();
	match stream.write(&bytes) {
		Ok(_) => {},
		Err(e) => panic!("{}", e)
	}
}

pub fn write_u32_le(stream: &mut Cursor<Vec<u8>>, val: u32) {
	let bytes = val.to_le_bytes();
	match stream.write(&bytes) {
		Ok(_) => {},
		Err(e) => panic!("{}", e)
	}
}

pub fn write_u64_le(stream: &mut Cursor<Vec<u8>>, val: u64) {
	let bytes = val.to_le_bytes();
	match stream.write(&bytes) {
		Ok(_) => {},
		Err(e) => panic!("{}", e)
	}
}

pub fn write_hex_le(stream: &mut Cursor<Vec<u8>>, val: String, with_varint: bool) {
	if with_varint { write_varint(stream, val.len() as u64 / 2) }
	let bytes = decode_hex_le(&val).expect("Something wrong with the hex");
	match stream.write(&bytes) {
		Ok(_) => {},
		Err(e) => panic!("{}", e)
	}
}

pub fn write_hex_be(stream: &mut Cursor<Vec<u8>>, val: String, with_varint: bool) {
	if with_varint { write_varint(stream, val.len() as u64 / 2) }
	let bytes = decode_hex_be(&val).expect("Something wrong with the hex");
	match stream.write(&bytes) {
		Ok(_) => {},
		Err(e) => panic!("{}", e)
	}
}

pub fn write_varint(stream: &mut Cursor<Vec<u8>>, size: u64) {
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

	match stream.write(&bytes) {
		Ok(_) => {},
		Err(e) => panic!("{}", e)
	}
}

// sometimes can't do i32::from_le_bytes because from_le_bytes requires a 4 byte input
// can convert 2 bytes to 4 bytes: https://dev.to/wayofthepie/three-bytes-to-an-integer-13g5

