use std::fs::File;
use std::io::Read;
use std::path::Path;

use bendy::decoding::{Object, ResultExt};

use super::*;

pub(crate) struct Torrent {
    pub(crate) meta_info: MetaInfo,
    pub(crate) info_hash: Sha1Digest,
}

impl Torrent {
    pub(crate) fn parse<P: AsRef<Path>>(path: P) -> Self {
        let mut file = File::open(path.as_ref()).unwrap_or_else(|_| panic!("Failed to open {:?}", path.as_ref()));
        let mut buffer = vec![];
        file.read_to_end(&mut buffer).expect("Failed to read file");
        let info_hash = info_hash(&buffer).unwrap();
        let meta_info: MetaInfo = bendy::serde::from_bytes(buffer.as_slice()).unwrap();
        Self {
            meta_info,
            info_hash,
        }
    }
}

fn info_hash<D: AsRef<[u8]>>(data: D) -> Result<Sha1Digest> {
    let mut decoder = bendy::decoding::Decoder::new(data.as_ref());
    let obj = decoder.next_object();
    if let Ok(Some(Object::Dict(mut meta_dict))) = obj {
        while let Ok(Some((name, obj))) = meta_dict.next_pair() {
            if String::from_utf8(name.to_owned()) == Ok("info".to_owned()) {
                let info_decoder = obj.dictionary_or(Err(
                    Error::BencodeDecode("info data type not dict".to_owned())
                ))?;
                let raw_info = info_decoder.into_raw().context("Failed to consume info")?;
                return Ok(Sha1Digest::from_data(raw_info));
            }
        }
    }
    Err(Error::BencodeDecode("Failed to calculate info hash".to_owned()))
}

#[cfg(test)]
mod tests {}