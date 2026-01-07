use std::fmt;

use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZedisString {
    Inline(u8, [u8; 22]), // 1 byte len, 22 bytes data (fits in 24 bytes total with discriminant)
    Heap(String),
}

impl ZedisString {
    pub fn new(s: &str) -> Self {
        let len = s.len();
        if len <= 22 {
            let mut buf = [0u8; 22];
            buf[..len].copy_from_slice(s.as_bytes());
            ZedisString::Inline(len as u8, buf)
        } else {
            ZedisString::Heap(s.to_string())
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            ZedisString::Inline(len, buf) => unsafe {
                std::str::from_utf8_unchecked(&buf[..*len as usize])
            },
            ZedisString::Heap(s) => s.as_str(),
        }
    }
    
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        match self {
            ZedisString::Inline(len, _) => *len as usize,
            ZedisString::Heap(s) => s.len(),
        }
    }
}

impl fmt::Display for ZedisString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
