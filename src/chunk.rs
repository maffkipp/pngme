use crate::chunk_type::ChunkType;
use crate::{Error, Result};
use crc::{Crc, CRC_32_ISO_HDLC};
use std::{
    fmt,
    io::{BufReader, Read},
};

const CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

#[derive(Debug)]
pub struct Chunk {
    length: u32,
    chunk_type: ChunkType,
    data: Vec<u8>,
    crc: u32,
}

impl Chunk {
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Self {
        let crc = calculate_crc(&chunk_type, &data);

        Chunk {
            length: data.len() as u32,
            chunk_type,
            data,
            crc,
        }
    }

    pub fn data_as_string(&self) -> Result<String> {
        let data = self.data.clone();
        match String::from_utf8(data) {
            Ok(data_string) => return Ok(data_string),
            Err(_) => Err("not valid utf8")?,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.length.to_be_bytes());
        bytes.extend_from_slice(&self.chunk_type.bytes());
        bytes.extend_from_slice(&self.data);
        bytes.extend_from_slice(&self.crc.to_be_bytes());

        bytes
    }

    pub fn length(&self) -> u32 {
        self.length
    }

    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn crc(&self) -> u32 {
        self.crc
    }
}

impl TryFrom<&Vec<u8>> for Chunk {
    type Error = Error;
    fn try_from(bytes: &Vec<u8>) -> Result<Self> {
        if bytes.len() < 12 {
            Err("Invalid chunk length")?;
        }

        // Create buffer reader and buffers for length and chunk type
        let mut reader = BufReader::new(&bytes[..]);
        let mut l_buf: [u8; 4] = [0; 4];
        let mut ct_buf: [u8; 4] = [0; 4];

        // Read values for length and chunk type off the front of the buffer
        reader.read_exact(&mut l_buf)?;
        reader.read_exact(&mut ct_buf)?;

        // Convert to correct types
        let length = u32::from_be_bytes(l_buf);
        let chunk_type = ChunkType::try_from(ct_buf)?;

        let data_terminator = length as usize + 8;

        // Slice off the data and crc from the input bytes
        let crc_bytes = bytes[data_terminator..data_terminator + 4].to_vec();
        let data = bytes[8..data_terminator].to_vec();
        
        // Bit shift operation to convert from list of bytes to u32
        let mut crc = 0;
        for (i, b) in crc_bytes.iter().enumerate() {
            let shift = (3 - i) * 8;
            crc = crc + ((*b as u32) << shift);
        }

        // Check CRC
        let test_crc = calculate_crc(&chunk_type, &data);
        if test_crc != crc {
            Err("CRC mismatch")?;
        }

        Ok(Chunk {
            length,
            chunk_type,
            data,
            crc,
        })
    }
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let data_as_strings: Vec<String> = self.data.iter().map(|n| n.to_string()).collect();
        let joined_data = data_as_strings.join("");
        write!(
            f,
            "{}{}{}{}",
            self.length, self.chunk_type, joined_data, self.crc
        )
    }
}

fn calculate_crc(chunk_type: &ChunkType, data: &Vec<u8>) -> u32 {
    let mut crc_bytes = Vec::from(chunk_type.bytes());
    crc_bytes.extend(data);
    CRC.checksum(&crc_bytes.as_slice())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!"
            .as_bytes()
            .to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());
        println!("CHUNK: {:?}", chunk);
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }
}
