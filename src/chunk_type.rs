use crate::types::{error_from, Error, Result};
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};

/// Represents a PNG chunk type code
#[derive(Eq, PartialEq, Debug)]
pub struct ChunkType {
    ancillary: u8,
    private: u8,
    reserved: u8,
    safe_to_copy: u8,
}

impl ChunkType {
    /// Returns the raw chunk type bytes
    pub fn bytes(&self) -> [u8; 4] {
        [
            self.ancillary,
            self.private,
            self.reserved,
            self.safe_to_copy,
        ]
    }

    /// Tests chunk type validity
    pub fn is_valid(&self) -> bool {
        self.is_reserved_bit_valid()
    }

    /// Tests chunk type ancillary bit (byte 0 bit 5)
    pub fn is_critical(&self) -> bool {
        self.ancillary & 32u8 == 0u8
    }

    /// Tests chunk type private bit (byte 1 bit 5)
    pub fn is_public(&self) -> bool {
        self.private & 32u8 == 0u8
    }

    /// Tests chunk type reserved bit validity (byte 2 bit 5)
    pub fn is_reserved_bit_valid(&self) -> bool {
        self.reserved & 32u8 == 0u8
    }

    /// Tests chunk type copy bit (byte 3 bit 5)
    pub fn is_safe_to_copy(&self) -> bool {
        self.safe_to_copy & 32u8 != 0u8
    }
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = Error;

    /// Gives the ability to construct a ChunkType from a [u8; 4]
    fn try_from(value: [u8; 4]) -> Result<Self> {
        for (i, byte) in value.iter().enumerate() {
            match byte {
                b'a'..=b'z' => {}
                b'A'..=b'Z' => {}
                _ => return Err(error_from(&format!("byte {} is out of range", i))),
            }
        }
        Ok(ChunkType {
            ancillary: value[0],
            private: value[1],
            reserved: value[2],
            safe_to_copy: value[3],
        })
    }
}

impl FromStr for ChunkType {
    type Err = Error;

    /// Gives the ability to  construct a ChunkType from a &str
    fn from_str(value: &str) -> Result<Self> {
        if value.len() != 4 {
            Err(error_from("`value` must be exactly 4 bytes long"))
        } else {
            let bytes: [u8; 4] = value.as_bytes().try_into()?;
            ChunkType::try_from(bytes)
        }
    }
}

impl Display for ChunkType {
    /// Gives the ability to format ChunkType as its ASCII equivalent
    /// and Enables ToString
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", String::from_utf8_lossy(&self.bytes()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_valid());

        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    pub fn test_chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}
