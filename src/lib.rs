#![no_std]

use core::fmt::Debug;

use hashbrown::HashMap;

use heapless::{String, Vec};

pub const MAX_BODY_SIZE: usize = 4 * 1024;

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
    headers: HashMap<String<128>, String<128>>,
    body: Option<String<MAX_BODY_SIZE>>,
}

impl Default for MessageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageBuilder {
    pub fn new() -> Self {
        Self {
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn add_header(mut self, key: String<128>, value: String<128>) -> Self {
        self.headers.insert(key, value);
        self
    }

    pub fn set_body(mut self, body: String<MAX_BODY_SIZE>) -> Self {
        self.body = Some(body);
        self
    }

    pub fn build(self) -> Option<Message> {
        Some(Message {
            header: String::from(
                self.headers
                    .into_iter()
                    .map(|(k, v)| {
                        let mut out: String<260> = String::new();
                        out.push_str(k.as_str()).ok()?;
                        out.push_str(": ").ok()?;
                        out.push_str(v.as_str()).ok()?;
                        Some(out)
                    })
                    .try_fold(String::new(), |mut v, b| {
                        v.push_str(&b?).ok()?;
                        Some(v)
                    })?,
            ),
            body: self.body.unwrap_or(String::new()),
        })
    }
}

#[derive(Clone, PartialEq)]
pub struct Message {
    pub header: String<MAX_BODY_SIZE>,
    pub body: String<MAX_BODY_SIZE>,
}

impl Debug for Message {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "[0x1][0x2][{}][{:?}][0x3][0x2][{:?}][0x4]",
            self.header.clone().into_bytes().len(),
            self.header,
            self.body
        ))
    }
}

impl Message {
    pub fn to_bytes(self) -> Option<Vec<u8, { MAX_BODY_SIZE * 2 }>> {
        let mut out = Vec::new();

        out.push(0x1).ok()?;
        out.push(0x2).ok()?;

        let header_len = u16_to_u8s(self.header.clone().into_bytes().len() as u16 + 1);

        out.push(header_len[0]).ok()?;
        out.push(header_len[1]).ok()?;

        out.extend_from_slice(self.header.as_bytes()).ok()?;

        out.push(0x0).ok()?;

        out.push(0x3).ok()?;
        out.push(0x2).ok()?;

        out.extend_from_slice(self.body.as_bytes()).ok()?;

        out.push(0x0).ok()?;

        out.push(0x4).ok()?;

        Some(out)
    }

    pub fn from_bytes(input: &[u8]) -> Option<Self> {
        if input[0] != 0x1 || input[1] != 0x2 || input[input.len() - 1] != 0x4 {
            return None;
        }

        let mut header_len_bytes = [0u8; 2];
        header_len_bytes.copy_from_slice(&input[2..4]);
        let header_len = u16::from_le_bytes(header_len_bytes) as usize;

        let header = String::from_iter(
            input[4..4 + header_len]
                .iter()
                .map(|&a| a as char)
                .take_while(|x| *x != '\0'),
        );

        let header_end = 4 + header_len;

        let body_start = input[header_end..].iter().position(|&a| a == 0x2)? + header_end;

        let body = String::from_iter(
            input[body_start + 1..input.len() - 2]
                .iter()
                .map(|&a| a as char),
        );

        let message = Self { header, body };

        Some(message)
    }
}

#[cfg(test)]
mod test {
    use core::str::FromStr;
    use std::println;

    use heapless::String;

    extern crate std;

    #[test]
    fn test_bytes() {
        let message = super::MessageBuilder::new()
            .add_header(
                String::from_str("Content-Type").unwrap(),
                String::from_str("text/html").unwrap(),
            )
            .set_body(String::from_str("<html><body><h1>Hello, world!</h1></body></html>").unwrap())
            .build()
            .unwrap();

        let bytes = message.clone().to_bytes().unwrap();
        println!("{:?}", bytes);
        let message2 = super::Message::from_bytes(&bytes).unwrap();

        assert_eq!(message, message2);
    }

    #[test]
    fn test_header() {
        let message = [
            1, 2, 20, 0, 82, 101, 113, 117, 101, 115, 116, 45, 68, 97, 116, 97, 58, 32, 112, 104,
            97, 115, 101, 115, 0, 3, 2, 0, 4, 0, 0, 2,
        ]
        .to_vec();

        let end = message.iter().position(|&a| a == 0x4).unwrap();

        let message = message[..end + 1].to_vec();

        let message = super::Message::from_bytes(&message);

        assert!(message.is_some());

        assert_eq!(
            message,
            Some(super::Message {
                header: String::from_str("Request-Data: phases").unwrap(),
                body: String::from_str("").unwrap(),
            })
        );
    }
}
