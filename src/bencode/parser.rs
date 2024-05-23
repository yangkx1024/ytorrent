//! Bencode parser.
//!
//! Example:
//! ```
//! use ytorrent::{BencodeParser, Object};
//! // "{"key1": "value", "key2": 123}" in Json format
//! let data = b"d4:key15:value4:key2i123ee";
//! let mut parser = BencodeParser::new(data);
//! let object = parser.parse().unwrap();
//! match object {
//!     Some(Object::Dict(mut decoder)) => {
//!         let (key1, value1) = decoder.next_pair().unwrap().unwrap();
//!         assert_eq!(key1, b"key1");
//!         if let Object::Bytes(bytes) = value1 {
//!             assert_eq!(bytes, b"value");
//!         } else {
//!             unreachable!()
//!         };
//!         drop(value1);
//!         let (key2, value2) = decoder.next_pair().unwrap().unwrap();
//!         assert_eq!(key2, b"key2");
//!         if let Object::Int(int_str) = value2 {
//!             assert_eq!(int_str.parse(), Ok(123));
//!         } else {
//!             unreachable!()
//!         };
//!         drop(value2);
//!         assert!(decoder.next_pair().unwrap().is_none());
//!     }
//!     _ => unreachable!()
//! }
//! ```
use std::rc::Rc;

use super::*;
use super::Error::*;

pub struct BencodeParser<'de> {
    pub(super) data: &'de [u8],
    pub(super) offset: usize,
    peeked_token: Option<Rc<Token<'de>>>,
}

impl<'de> BencodeParser<'de> {
    pub fn new(data: &'de [u8]) -> Self {
        BencodeParser {
            data,
            offset: 0,
            peeked_token: None,
        }
    }

    /// Peek the next token, but not consume it.
    ///
    /// See [Self::next_token]
    pub(super) fn peek_token(&mut self) -> Result<Rc<Token<'de>>> {
        // Consume the cached token first
        if let Some(token) = &self.peeked_token {
            println!("peek reused token: {}", token);
            return Ok(token.clone());
        }
        self.next_raw_token().map(|token| {
            println!("peek token: {}", token);
            let token = Rc::new(token);
            self.peeked_token = Some(token.clone());
            token
        })
    }

    /// Consume next token.
    ///
    /// See [`Self::peek_token`]
    pub(super) fn next_token(&mut self) -> Result<Rc<Token<'de>>> {
        // Consume the cached token first
        if let Some(token) = self.peeked_token.take() {
            println!("reused token: {}", token);
            return Ok(token);
        }
        self.next_raw_token().map(Rc::new)
    }

    /// Try to parse next token
    fn next_raw_token(&mut self) -> Result<Token<'de>> {
        let position = self.offset;
        match self.take_byte().ok_or(BencodeDecode(format!(
            "unexpected EOF at {} when parse token",
            position
        )))? as char
        {
            'e' => Ok(Token::End),
            'l' => Ok(Token::List),
            'd' => Ok(Token::Dict),
            'i' => Ok(Token::Num(self.take_int('e')?)),
            c if c.is_ascii_digit() => {
                self.offset -= 1;
                Ok(Token::String(self.take_bytes()?))
            }
            tok => Err(BencodeDecode(format!(
                "invalid token {} at {}",
                tok, self.offset
            ))),
        }
        .map(|token| {
            println!("parsed token: {}", token);
            token
        })
    }

    /// Except next token is "d"
    pub(super) fn expect_dict_begin(&mut self, log: &str) -> Result<()> {
        let position = self.offset;
        match &*self.next_token()? {
            Token::Dict => Ok(()),
            other => Err(SerdeCustom(format!(
                "expect dict for {} but get {} at {}",
                log, other, position
            ))),
        }
    }

    /// Except next token is "l"
    pub(super) fn expect_list_begin(&mut self, log: &str) -> Result<()> {
        let position = self.offset;
        match &*self.next_token()? {
            Token::List => Ok(()),
            other => Err(SerdeCustom(format!(
                "expect list for {} but get {} at {}",
                log, other, position
            ))),
        }
    }

    /// Except next token is "e"
    pub(super) fn expect_end(&mut self, log: &str) -> Result<()> {
        let position = self.offset;
        match &*self.next_token()? {
            Token::End => Ok(()),
            other => Err(SerdeCustom(format!(
                "expect end for {} but get {} at {}",
                log, other, position
            ))),
        }
    }

    /// Except next two tokens are "l" and "e"
    pub(super) fn expect_empty_list(&mut self, log: &str) -> Result<()> {
        self.expect_list_begin(log)?;
        self.expect_end(log)?;
        Ok(())
    }

    /// Move forward for one byte
    fn take_byte(&mut self) -> Option<u8> {
        if self.offset < self.data.len() {
            let ret = Some(self.data[self.offset]);
            self.offset += 1;
            ret
        } else {
            None
        }
    }

    /// Move forward `count` bytes
    fn take_chunk(&mut self, count: usize) -> Option<&'de [u8]> {
        match self.offset.checked_add(count) {
            Some(end_pos) if end_pos <= self.data.len() => {
                let ret = &self.data[self.offset..end_pos];
                self.offset = end_pos;
                Some(ret)
            }
            _ => None,
        }
    }

    /// Move forward to next `expected_terminator`
    fn take_int(&mut self, expected_terminator: char) -> Result<&'de str> {
        enum State {
            Start,
            Sign,
            Zero,
            Digits,
        }

        let mut cur_position = self.offset;
        let mut state = State::Start;

        let mut success = false;
        while cur_position < self.data.len() {
            let c = self.data[cur_position] as char;
            match state {
                State::Start => {
                    if c == '-' {
                        state = State::Sign;
                    } else if c == '0' {
                        state = State::Zero;
                    } else if ('1'..='9').contains(&c) {
                        state = State::Digits;
                    } else {
                        return Err(BencodeDecode(format!(
                            "expect '-' or digit but get {} , at {}",
                            c, cur_position
                        )));
                    }
                }
                State::Zero => {
                    if c == expected_terminator {
                        success = true;
                        break;
                    } else {
                        return Err(BencodeDecode(format!(
                            "expect {} but get {}, at {}",
                            expected_terminator, c, cur_position
                        )));
                    }
                }
                State::Sign => {
                    if ('1'..='9').contains(&c) {
                        state = State::Digits;
                    } else {
                        return Err(BencodeDecode(format!(
                            "except sign but get {}, at {}",
                            c, cur_position
                        )));
                    }
                }
                State::Digits => {
                    if c.is_ascii_digit() {
                        // do nothing, this is ok
                    } else if c == expected_terminator {
                        success = true;
                        break;
                    } else {
                        return Err(BencodeDecode(format!(
                            "expect digit bug get {}, at {}",
                            c, cur_position
                        )));
                    }
                }
            }
            cur_position += 1;
        }

        if !success {
            return Err(BencodeDecode(format!("unexpected EOF at {}", cur_position)));
        }

        let slice = &self.data[self.offset..cur_position];
        self.offset = cur_position + 1;
        let str = unsafe { std::str::from_utf8_unchecked(slice) };
        Ok(str)
    }

    /// Move forward to end of bytes.
    ///
    /// Before:
    ///
    /// ```text
    /// "d2:xxe"
    ///  -^----
    /// ```
    ///
    /// After:
    ///
    /// ```text
    /// "d2:xxe"
    ///  _____^
    /// ```
    fn take_bytes(&mut self) -> Result<&'de [u8]> {
        let cur_position = self.offset;
        let int_str = self.take_int(':')?;
        let len = int_str
            .parse::<usize>()
            .map_err(|_| BencodeDecode(format!("invalid integer at {}", cur_position)))?;
        self.take_chunk(len).ok_or(BencodeDecode(format!(
            "unexpected EOF at {} when read bytes",
            self.offset
        )))
    }

    /// Parse raw bencode bytes to [Object].
    pub fn parse<'obj>(&'obj mut self) -> Result<Option<Object<'obj, 'de>>> {
        match *self.next_token()? {
            Token::List => Ok(Some(Object::List(ListDecoder::new(self)))),
            Token::Dict => Ok(Some(Object::Dict(DictDecoder::new(self)))),
            Token::Num(str) => Ok(Some(Object::Int(str))),
            Token::String(bytes) => Ok(Some(Object::Bytes(bytes))),
            Token::End => Ok(None),
        }
    }
}
