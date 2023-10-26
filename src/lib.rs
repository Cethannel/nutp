#![no_std]

#[macro_use]
extern crate alloc;

use core::fmt::Debug;

use alloc::{ffi::CString, string::String, vec::Vec};
use hashbrown::HashMap;

/// Converts a u16 to array of 2 u8s corresponding to the upper and lower 8
/// bits respectively
///
/// # Arguments
///
/// * `input` - The u16 input that will be split.
///
/// # Return value
///
/// A array of exactly 2 u8s which correspond to the upper and lower 8 bits of
/// `input`
const fn u16_to_u8s(input: u16) -> [u8; 2] {
    [
        (input & (u8::MAX as u16)) as u8,
        ((input >> 8) & (u8::MAX as u16)) as u8,
    ]
}

#[derive(Debug)]
pub struct MessageBuilder {
    headers: HashMap<CString, CString>,
    body: Option<CString>,
}

impl MessageBuilder {
    pub fn new() -> Self {
        Self {
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn add_header(mut self, key: CString, value: CString) -> Self {
        self.headers.insert(key, value);
        self
    }

    pub fn set_body(mut self, body: CString) -> Self {
        self.body = Some(body);
        self
    }

    pub fn build(self) -> Option<Message> {
        Some(Message {
            header: CString::new(
                self.headers
                    .into_iter()
                    .map(|(k, v)| Some(format!("{}: {}", k.to_str().ok()?, v.to_str().ok()?)))
                    .try_fold(String::new(), |v, b| Some(format!("{v}{}", b?)))?,
            )
            .ok()?,
            body: self.body.unwrap_or(CString::new("").unwrap()),
        })
    }
}

#[derive(Clone)]
pub struct Message {
    header: CString,
    body: CString,
}

impl Debug for Message {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "[0x1][0x2][{}][{:?}][0x3][0x2][{:?}][0x4]",
            (&self.header).to_bytes().len(),
            self.header,
            self.body
        ))
    }
}

impl Message {
    pub fn to_bytes(self) -> Vec<u8> {
        vec![
            vec![0x1, 0x2],
            u16_to_u8s(self.header.to_bytes().len() as u16).to_vec(),
            self.header.to_bytes().to_vec(),
            vec![0x3, 0x2],
            self.body.to_bytes().to_vec(),
            vec![0x4],
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}
