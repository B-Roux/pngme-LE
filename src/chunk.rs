use crate::{
    chunk_type::ChunkType,
    types::{assert_or_err, error_from, Error, Result},
};
use std::fmt::{Display, Formatter, Result as FmtResult};

// fixed length field widths
pub const LENGTH_WIDTH: usize = 4;
pub const TYPE_WIDTH: usize = 4;
pub const CRC_WIDTH: usize = 4;
pub const REQ_FIELDS_WIDTH: usize = LENGTH_WIDTH + TYPE_WIDTH + CRC_WIDTH;

/// Stores a PNG chunk
#[derive(Debug)]
pub struct Chunk {
    chunk_type: ChunkType,
    data: Vec<u8>,
}

impl Chunk {
    /// Create a new chunk from a type and associated data
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Chunk {
        Chunk { chunk_type, data }
    }

    /// Get the length of the data portion of this chunk
    pub fn length(&self) -> u32 {
        self.data.len() as u32
    }

    /// Get the type of this chunk
    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    /// Get the data portion associated with this chunk
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Calculate the checksum of this chunk based on its type and data portion
    pub fn crc(&self) -> u32 {
        let crc = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);

        let mut bytes = Vec::with_capacity((self.length() as usize) + TYPE_WIDTH);
        bytes.extend(self.chunk_type.bytes());
        bytes.extend(self.data());

        crc.checksum(&bytes)
    }

    /// Try to read data as UTF8
    pub fn data_as_string(&self) -> Result<String> {
        match String::from_utf8(self.data.clone()) {
            Ok(s) => Ok(s),
            Err(_) => Err(error_from("chunk data is not valid utf8")),
        }
    }

    /// Get this entire chunk as a vector of raw bytes
    pub fn as_bytes(&self) -> Vec<u8> {
        // I could use iterators here, but I like this better - it feels simpler to me
        let mut bytes = Vec::with_capacity(REQ_FIELDS_WIDTH + (self.length() as usize));
        bytes.extend(self.length().to_be_bytes());
        bytes.extend(self.chunk_type.bytes());
        bytes.extend(self.data());
        bytes.extend(self.crc().to_be_bytes());
        bytes
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = Error;
    /// Gives the ability to construct a Chunk from raw bytes
    fn try_from(value: &[u8]) -> Result<Self> {
        let length_begin: usize = 0;
        let type_begin: usize = length_begin + LENGTH_WIDTH;
        let data_begin: usize = type_begin + TYPE_WIDTH;

        // read the length
        assert_or_err(
            value.len() >= REQ_FIELDS_WIDTH, 
            "invalid chunk data (incomplete)",
        )?;
        let chunk_length = u32::from_be_bytes(value[length_begin..type_begin].try_into()?);

        // make sure the slice length matches the indicated length
        assert_or_err(
            value.len() == REQ_FIELDS_WIDTH + (chunk_length as usize),
            "invalid chunk data (invalid length)",
        )?;
        let crc_begin = data_begin + (chunk_length as usize);

        // read remaining fields
        let chunk_type_bytes: [u8; 4] = value[type_begin..data_begin].try_into()?;
        let chunk_type = ChunkType::try_from(chunk_type_bytes)?;
        let chunk_data = value[data_begin..crc_begin].to_vec();
        let chunk_crc_bytes: [u8; 4] = value[crc_begin..].try_into()?;
        let chunk_crc = u32::from_be_bytes(chunk_crc_bytes);
        // validate & return
        let unchecked_chunk = Chunk::new(chunk_type, chunk_data);
        assert_or_err(
            unchecked_chunk.crc() == chunk_crc, 
            "checksum does not match data",
        )?;
        Ok(unchecked_chunk)
    }
}

impl Display for Chunk {
    /// Gives the ability to format ChunkType as a string
    /// and Enables ToString
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let length = self.length();
        let type_ = self.chunk_type();
        let crc = self.crc();
        writeln!(
            f, 
            "Chunk {{Length: {}, Type: {}, Crc: {}}}",
            length, type_, crc
        )?;
        Ok(())
    }
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
