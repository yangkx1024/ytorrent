use std::fmt::{Display, Formatter};

use super::*;

pub enum Object<'obj, 'de: 'obj> {
    Int(&'de str),
    Bytes(&'de [u8]),
    Dict(DictDecoder<'obj, 'de>),
    List(ListDecoder<'obj, 'de>),
}

impl<'obj, 'de: 'obj> Display for Object<'obj, 'de> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Int(str) => write!(f, "Integer {str}"),
            Object::Bytes(bytes) => write!(f, "Bytes({})", bytes.len()),
            Object::Dict(_) => write!(f, "Dict"),
            Object::List(_) => write!(f, "List"),
        }
    }
}

impl<'obj, 'de: 'obj> Object<'obj, 'de> {
    pub(crate) fn unwrap_bytes(self) -> Option<&'de [u8]> {
        match self {
            Object::Bytes(bytes) => Some(bytes),
            _ => None,
        }
    }
}

/// Decode list struct of bencoded data
pub struct ListDecoder<'obj, 'de: 'obj> {
    parser: &'obj mut BencodeParser<'de>,
    finished: bool,
    start_point: usize,
}

impl<'obj, 'de: 'obj> ListDecoder<'obj, 'de> {
    pub(super) fn new(parser: &'obj mut BencodeParser<'de>) -> Self {
        let start_point = parser.offset - 1;
        ListDecoder {
            parser,
            finished: false,
            start_point,
        }
    }

    pub fn next_object<'item>(&'item mut self) -> Result<Option<Object<'item, 'de>>> {
        if self.finished {
            return Ok(None);
        }

        let item = self.parser.parse()?;

        if item.is_none() {
            self.finished = true;
        }

        Ok(item)
    }

    fn consume_all(&mut self) -> Result<()> {
        while self.next_object()?.is_some() {
            // just drop the items
        }
        Ok(())
    }
}

impl<'obj, 'de: 'obj> TryFrom<ListDecoder<'obj, 'de>> for &'de [u8] {
    type Error = Error;

    fn try_from(mut value: ListDecoder<'obj, 'de>) -> Result<Self> {
        value.consume_all()?;
        Ok(&value.parser.data[value.start_point..value.parser.offset])
    }
}

impl<'obj, 'de: 'obj> Drop for ListDecoder<'obj, 'de> {
    fn drop(&mut self) {
        // we don't care about errors in drop; they'll be reported again in the parent
        self.consume_all().ok();
    }
}

pub struct DictDecoder<'obj, 'de: 'obj> {
    parser: &'obj mut BencodeParser<'de>,
    finished: bool,
    start_point: usize,
}

impl<'obj, 'de: 'obj> DictDecoder<'obj, 'de> {
    pub(super) fn new(parser: &'obj mut BencodeParser<'de>) -> Self {
        let start_point = parser.offset - 1;
        DictDecoder {
            parser,
            finished: false,
            start_point,
        }
    }

    pub fn next_pair<'item>(&'item mut self) -> Result<Option<(&'de [u8], Object<'item, 'de>)>> {
        if self.finished {
            return Ok(None);
        }

        let key = self.parser.parse()?.and_then(Object::unwrap_bytes);

        if let Some(k) = key {
            let position = self.parser.offset;
            let v = self.parser.parse()?.ok_or(Error::BencodeDecode(format!(
                "unexpected end of list at {}",
                position
            )))?;
            Ok(Some((k, v)))
        } else {
            // We can't have gotten anything but a string, as anything else would be
            // a state error
            self.finished = true;
            Ok(None)
        }
    }

    fn consume_all(&mut self) -> Result<()> {
        while self.next_pair()?.is_some() {
            // just drop the items
        }
        Ok(())
    }
}

impl<'obj, 'de: 'obj> TryFrom<DictDecoder<'obj, 'de>> for &'de [u8] {
    type Error = Error;

    fn try_from(mut value: DictDecoder<'obj, 'de>) -> Result<Self> {
        value.consume_all()?;
        Ok(&value.parser.data[value.start_point..value.parser.offset])
    }
}

impl<'obj, 'de: 'obj> Drop for DictDecoder<'obj, 'de> {
    fn drop(&mut self) {
        // we don't care about errors in drop; they'll be reported again in the parent
        self.consume_all().ok();
    }
}
