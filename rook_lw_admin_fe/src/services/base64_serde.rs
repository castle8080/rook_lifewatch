//! Helper module for base64 serialization of byte vectors
//! 
//! Use with serde's `with` attribute:
//! ```rust
//! #[derive(Serialize, Deserialize)]
//! struct MyStruct {
//!     #[serde(with = "base64_serde")]
//!     data: Vec<u8>,
//! }
//! ```

use serde::{Deserialize, Deserializer, Serializer};
use base64::{Engine as _, engine::general_purpose::STANDARD};

pub fn serialize<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&STANDARD.encode(bytes))
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    STANDARD.decode(s).map_err(serde::de::Error::custom)
}
