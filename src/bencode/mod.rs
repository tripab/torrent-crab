//! Bencode encoding and decoding
//!
//! Bencode is the encoding used by BitTorrent for storing and transmitting
//! loosely structured data. It supports four data types:
//! - Byte strings
//! - Integers
//! - Lists
//! - Dictionaries

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A bencode value that can be encoded/decoded
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Bytes(#[serde(with = "serde_bytes")] Vec<u8>),
    Int(i64),
    List(Vec<Value>),
    Dict(BTreeMap<String, Value>),
}

impl Value {
    /// Decode bencode data into a Value
    pub fn decode(data: &[u8]) -> crate::Result<Self> {
        serde_bencode::from_bytes(data).map_err(|e| crate::Error::BencodeDecode(e.to_string()))
    }

    /// Encode a Value into bencode format
    pub fn encode(&self) -> crate::Result<Vec<u8>> {
        serde_bencode::to_bytes(self).map_err(|e| crate::Error::BencodeEncode(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_integer() {
        let data = b"i42e";
        let value = Value::decode(data).unwrap();
        assert_eq!(value, Value::Int(42));
    }

    #[test]
    fn test_decode_string() {
        let data = b"4:spam";
        let value = Value::decode(data).unwrap();
        assert_eq!(value, Value::Bytes(b"spam".to_vec()));
    }

    #[test]
    fn test_decode_list() {
        let data = b"l4:spami42ee";
        let value = Value::decode(data).unwrap();
        if let Value::List(list) = value {
            assert_eq!(list.len(), 2);
            assert_eq!(list[0], Value::Bytes(b"spam".to_vec()));
            assert_eq!(list[1], Value::Int(42));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let original = Value::Int(123);
        let encoded = original.encode().unwrap();
        let decoded = Value::decode(&encoded).unwrap();
        assert_eq!(original, decoded);
    }
}
