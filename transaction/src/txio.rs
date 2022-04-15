use std::fmt::write;
use std::io::{Read, Cursor, Seek, SeekFrom};
use std::num::ParseIntError;

// ---- Conversions ----

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

fn get_decimal_value_u8(bytes: &[u8; 1]) -> u8 {
	return u8::from_le_bytes(*bytes)
    // ((bytes[0] as u8) <<  0) +
}

fn get_decimal_value_u16(bytes: &[u8; 2]) -> u16 {
	u16::from_le_bytes(*bytes)
    // ((bytes[0] as u16) <<  0) +
	// ((bytes[1] as u16) <<  8)
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

fn get_decimal_value_u64(bytes: &[u8]) -> u64 {
	return u64::from_le_bytes(bytes.try_into().unwrap());
}

fn get_decimal_value_i64(bytes: &[u8]) -> i64 {
	return i64::from_le_bytes(bytes.try_into().unwrap());
}

// ---- Buffer reading ----

pub fn read_u8(stream: &mut Cursor<Vec<u8>>) -> u8 {
	let mut bytes = [0; 1];
	match stream.read(&mut bytes) {
		Ok(_) => get_decimal_value_u8(&bytes),
		Err(e) => panic!("{}", e)
	}
}

pub fn read_u16(stream: &mut Cursor<Vec<u8>>) -> u16 {
	let mut bytes = [0; 2];
	match stream.read(&mut bytes) {
		Ok(_) => get_decimal_value_u16(&bytes),
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

pub fn read_u32(stream: &mut Cursor<Vec<u8>>) -> u32 {
	let mut bytes = [0; 4];
	match stream.read(&mut bytes) {
		Ok(_) => get_decimal_value_u32(&bytes),
		Err(e) => panic!("{}", e)
	}
}

pub fn read_u64(stream: &mut Cursor<Vec<u8>>) -> u64 {
	let mut bytes = [0; 8];
	match stream.read(&mut bytes) {
		Ok(_) => get_decimal_value_u64(&bytes),
		Err(e) => panic!("{}", e)
	}
}

pub fn read_hex32(stream: &mut Cursor<Vec<u8>>) -> String {
	let mut bytes = [0; 4];
	match stream.read(&mut bytes) {
		Ok(_) => encode_hex(&bytes),
		Err(e) => panic!("{}", e)
	}
}

pub fn read_hex256(stream: &mut Cursor<Vec<u8>>) -> String {
	let mut bytes = [0; 32];
	match stream.read(&mut bytes) {
		Ok(_) => encode_hex(&bytes),
		Err(e) => panic!("{}", e)
	}
}

pub fn read_hex_var(stream: &mut Cursor<Vec<u8>>, length: u64) -> String {
	let mut bytes = vec![0; length as usize];
	match stream.read(&mut bytes) {
		Ok(_) => encode_hex(&bytes),
		Err(e) => panic!("{}", e)
	}
}

pub fn unread(stream: &mut Cursor<Vec<u8>>, length: i64) {
	match stream.seek(SeekFrom::Current(length)) {
		Ok(_) => (),
		Err(e) => panic!("{}", e)
	}
}
