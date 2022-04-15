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

// ---- Buffer reading ----

/**
 * Compact Size
 * size <  253        -- 1 byte
 * size <= USHRT_MAX  -- 3 bytes  (253 + 2 bytes)
 * size <= UINT_MAX   -- 5 bytes  (254 + 4 bytes)
 * size >  UINT_MAX   -- 9 bytes  (255 + 8 bytes)
*/
pub fn read_compact_size(stream: &mut Cursor<Vec<u8>>) -> u64 {
	let  varint_size: u8 = read_u8(stream);
	let size: u64;

	if varint_size < 253 {
		size = varint_size as u64;
	} else if varint_size == 253 {
		size = read_u16(stream) as u64;
		assert!(size > 253);
	} else if varint_size == 254 {
		size = read_u32(stream) as u64;
		assert!(size > 0x10000);
	} else if varint_size == 255 {
		size = read_u64(stream);
		assert!(size > 0x100000000);
	} else {
		panic!()
	}

	assert!(size != 0);
	size
}

pub fn read_u8(stream: &mut Cursor<Vec<u8>>) -> u8 {
	let mut bytes = [0; 1];
	match stream.read(&mut bytes) {
		Ok(_) => u8::from_le_bytes(bytes),
		Err(e) => panic!("{}", e)
	}
}

pub fn read_u16(stream: &mut Cursor<Vec<u8>>) -> u16 {
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

pub fn read_u32(stream: &mut Cursor<Vec<u8>>) -> u32 {
	let mut bytes = [0; 4];
	match stream.read(&mut bytes) {
		Ok(_) => u32::from_le_bytes(bytes),
		Err(e) => panic!("{}", e)
	}
}

pub fn read_u64(stream: &mut Cursor<Vec<u8>>) -> u64 {
	let mut bytes = [0; 8];
	match stream.read(&mut bytes) {
		Ok(_) => u64::from_le_bytes(bytes),
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
