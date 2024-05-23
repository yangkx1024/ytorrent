use std::fs::File;
use std::io::Read;
use std::path::Path;

use super::*;

/// Parsed torrent file
pub struct Torrent {
    pub meta_info: MetaInfo,
    pub info_hash: Sha1Digest,
}

impl Torrent {
    /// Parse torrent file to rust struct
    pub(crate) fn parse<P: AsRef<Path>>(path: P) -> Self {
        let mut file = File::open(path.as_ref())
            .unwrap_or_else(|_| panic!("Failed to open {:?}", path.as_ref()));
        let mut buffer = vec![];
        file.read_to_end(&mut buffer).expect("Failed to read file");
        let info_hash = info_hash(&buffer).unwrap();
        let meta_info: MetaInfo = de::from_bytes(&buffer).unwrap();
        Self {
            meta_info,
            info_hash,
        }
    }
}

fn info_hash<D: AsRef<[u8]>>(data: D) -> Result<Sha1Digest> {
    let mut decoder = BencodeParser::new(data.as_ref());
    let obj = decoder.parse()?;
    if let Some(Object::Dict(mut meta_dict)) = obj {
        while let Some((name, obj)) = meta_dict.next_pair()? {
            if std::str::from_utf8(name) == Ok("info") {
                return if let Object::Dict(info_decoder) = obj {
                    let raw_info: &[u8] = info_decoder.try_into()?;
                    Ok(Sha1Digest::digest(raw_info))
                } else {
                    Err(Error::BencodeDecode("info data type not dict".to_string()))
                };
            }
        }
    }
    Err(Error::BencodeDecode(
        "Failed to calculate info hash".to_string(),
    ))
}
