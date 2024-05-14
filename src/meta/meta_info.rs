use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use serde_with::rust::unwrap_or_skip;

use super::*;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct MetaInfo {
    /// The URL of the tracker.
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        with = "unwrap_or_skip"
    )]
    pub(crate) announce: Option<String>,
    /// [BEP-0012](https://www.bittorrent.org/beps/bep_0012.html) extends BitTorrent to support
    /// multiple trackers
    #[serde(
        rename = "announce-list",
        skip_serializing_if = "Option::is_none",
        default,
        with = "unwrap_or_skip"
    )]
    pub(crate) announce_list: Option<Vec<Vec<String>>>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        with = "unwrap_or_skip"
    )]
    pub(crate) comment: Option<String>,
    #[serde(
        rename = "created by",
        skip_serializing_if = "Option::is_none",
        default,
        with = "unwrap_or_skip"
    )]
    pub(crate) created_by: Option<String>,
    #[serde(
        rename = "creation date",
        skip_serializing_if = "Option::is_none",
        default,
        with = "unwrap_or_skip"
    )]
    pub(crate) creation_date: Option<u64>,
    pub(crate) info: Info,
    /// [BEP-0005](https://www.bittorrent.org/beps/bep_0005.html#entropy)
    /// DHT support
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        with = "unwrap_or_skip"
    )]
    pub(crate) nodes: Option<Vec<Node>>,
    #[serde(
        rename = "url-list",
        skip_serializing_if = "Option::is_none",
        default,
        with = "unwrap_or_skip"
    )]
    pub(crate) url_list: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Info {
    /// Single or Multiple files
    #[serde(flatten)]
    pub(crate) mode: FileMode,
    /// The name key maps to a UTF-8 encoded string which is the suggested name to save the file
    /// (or directory) as. It is purely advisory.
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        with = "unwrap_or_skip"
    )]
    pub(crate) name: Option<String>,
    /// piece length maps to the number of bytes in each piece the file is split into. For the
    /// purposes of transfer, files are split into fixed-size pieces which are all the same length
    /// except for possibly the last one which may be truncated. piece length is almost always a
    /// power of two, most commonly 2 18 = 256 K (BitTorrent prior to version 3.2 uses 2 20 = 1 M
    /// as default).
    #[serde(rename = "piece length")]
    pub(crate) piece_length: u64,
    /// pieces maps to a string whose length is a multiple of 20. It is to be subdivided into
    /// strings of length 20, each of which is the SHA1 hash of the piece at the corresponding index.
    pub(crate) pieces: PieceList,
    /// [BEP-0027](https://www.bittorrent.org/beps/bep_0027.html)
    /// extends BitTorrent to support private torrents.
    /// When generating a metainfo file, users denote a torrent as private by including the
    /// key-value pair "private=1" in the "info" dict of the torrent's metainfo file
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        with = "unwrap_or_skip"
    )]
    pub(crate) private: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(untagged)]
pub(crate) enum FileMode {
    Single {
        length: u64,
    },
    Multiple {
        files: Vec<FileInfo>,
    },
}

#[derive(Debug, PartialEq)]
pub(crate) struct PieceList(
    /// SHA-1 digest
    Vec<Sha1Digest>
);

impl Serialize for PieceList {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> where S: Serializer {
        let mut bytes = Vec::with_capacity(self.0.len() * Sha1Digest::LENGTH);

        for piece in &self.0 {
            bytes.extend_from_slice(piece.as_ref());
        }

        serde_bytes::Bytes::new(&bytes).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PieceList {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error> where D: Deserializer<'de> {
        let bytes = serde_bytes::ByteBuf::deserialize(deserializer)?.into_vec();
        if bytes.len() % Sha1Digest::LENGTH != 0 {
            return Err(D::Error::custom(format!(
                "buffer length {} is not a multiple of {}",
                bytes.len(),
                Sha1Digest::LENGTH
            )));
        }

        let digest_list = bytes
            .chunks_exact(Sha1Digest::LENGTH)
            .map(|chunk| Sha1Digest::new(chunk.try_into().unwrap()))
            .collect();

        Ok(Self(digest_list))
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub(crate) struct FileInfo {
    pub(crate) length: u64,
    pub(crate) path: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub(crate) struct Node(String, u16);

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;

    use bendy::encoding::ToBencode;

    use crate::meta::meta_info::{FileMode, Info, MetaInfo, Node, PieceList, Sha1Digest};

    const TAG_INFO: &str = "info";
    const TAG_LENGTH: &str = "length";
    const TAG_NAME: &str = "name";
    const TAG_PIECE_LENGTH: &str = "piece length";
    const TAG_PIECES: &str = "pieces";
    const TAG_ANNOUNCE: &str = "announce";
    const TAG_ANNOUNCE_LIST: &str = "announce-list";
    const TAG_NODES: &str = "nodes";
    const TAG_PRIVATE: &str = "private";

    const SAMPLE_NAME: &str = "test-name";
    const SAMPLE_SHA1_DIGEST: [u8; 20] = [
        0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
    ];
    const SAMPLE_ANNOUNCE: &str = "http://bttracker.debian.org:6969/announce";
    const SAMPLE_RAW_ANNOUNCE_LIST: &str = concat!(
    "l",
    "l",
    "41:http://bttracker.debian.org:6969/announce41:http://bttracker.debian.org:6969/announce",
    "e",
    "l",
    "41:http://bttracker.debian.org:6969/announce41:http://bttracker.debian.org:6969/announce",
    "e",
    "e",
    );
    const SAMPLE_RAW_NODES: &str = concat!(
    "l",
    "l",
    "9:127.0.0.1",
    "i6881e",
    "e",
    "l",
    "16:your.router.node",
    "i4804e",
    "e",
    "l",
    "34:2001:db8:100:0:d5c8:db3f:995e:c0f7",
    "i1941e",
    "e",
    "e",
    );


    #[test]
    fn test_se_de_piece_list() {
        let piece_list = PieceList(vec![Sha1Digest::new(SAMPLE_SHA1_DIGEST.to_owned())]);
        let sha1 = bendy::serde::to_bytes(&piece_list).unwrap();
        let piece_list: PieceList = bendy::serde::from_bytes(sha1.as_slice()).unwrap();
        assert_eq!(piece_list.0.first().unwrap().as_ref(), SAMPLE_SHA1_DIGEST);
    }


    fn build_info_data() -> Vec<u8> {
        let mut info: Vec<u8> = vec![];
        info.push(b'd');
        info.extend(TAG_LENGTH.to_bencode().unwrap());
        info.extend(1024.to_bencode().unwrap());
        info.extend(TAG_NAME.to_bencode().unwrap());
        info.extend(SAMPLE_NAME.to_bencode().unwrap());
        info.extend(TAG_PIECE_LENGTH.to_bencode().unwrap());
        info.extend(4096.to_bencode().unwrap());
        info.extend(TAG_PIECES.to_bencode().unwrap());
        let piece_list = PieceList(vec![Sha1Digest::new(SAMPLE_SHA1_DIGEST.to_owned())]);
        info.extend(bendy::serde::to_bytes(&piece_list).unwrap());
        info.extend(TAG_PRIVATE.to_bencode().unwrap());
        info.extend(0.to_bencode().unwrap());
        info.push(b'e');
        info
    }

    #[test]
    fn test_info_struct() {
        let info = build_info_data();
        let ret: Info = bendy::serde::from_bytes(info.as_slice()).unwrap();
        assert_eq!(ret.mode, FileMode::Single { length: 1024 });
        assert_eq!(ret.name, Some(SAMPLE_NAME.to_owned()));
        assert_eq!(ret.piece_length, 4096);
        assert_eq!(ret.pieces, PieceList(vec![Sha1Digest::new(SAMPLE_SHA1_DIGEST.to_owned())]));
        assert_eq!(ret.private, Some(false));
    }

    #[test]
    fn test_meta_announce() {
        let mut meta: Vec<u8> = vec![];
        meta.push(b'd');
        meta.extend(TAG_ANNOUNCE.to_bencode().unwrap());
        meta.extend(SAMPLE_ANNOUNCE.to_bencode().unwrap());
        meta.extend(TAG_ANNOUNCE_LIST.to_bencode().unwrap());
        meta.extend(SAMPLE_RAW_ANNOUNCE_LIST.as_bytes());
        meta.extend(TAG_INFO.to_bencode().unwrap());
        meta.extend(build_info_data().as_slice());
        meta.extend(TAG_NODES.to_bencode().unwrap());
        meta.extend(SAMPLE_RAW_NODES.as_bytes());
        meta.push(b'e');

        let ret: MetaInfo = bendy::serde::from_bytes(meta.as_slice()).unwrap();
        assert_eq!(ret.announce, Some(SAMPLE_ANNOUNCE.to_owned()));
        assert_eq!(
            ret.announce_list,
            Some(vec![
                vec![
                    "http://bttracker.debian.org:6969/announce".to_owned(),
                    "http://bttracker.debian.org:6969/announce".to_owned(),
                ],
                vec![
                    "http://bttracker.debian.org:6969/announce".to_owned(),
                    "http://bttracker.debian.org:6969/announce".to_owned(),
                ]
            ]),
        );
        assert_eq!(
            ret.nodes,
            Some(vec![
                Node("127.0.0.1".to_owned(), 6881),
                Node("your.router.node".to_owned(), 4804),
                Node("2001:db8:100:0:d5c8:db3f:995e:c0f7".to_owned(), 1941),
            ])
        )
    }

    #[test]
    fn test_decode_debian_torrent() {
        let mut file = File::open("./resources/debian-12.5.0-amd64-netinst.iso.torrent").unwrap();
        let mut buffer = vec![];
        file.read_to_end(&mut buffer).expect("Failed to read file");
        let meta: MetaInfo = bendy::serde::from_bytes(buffer.as_slice()).unwrap();
        assert_eq!(meta.announce, Some("http://bttracker.debian.org:6969/announce".to_owned()));
        assert_eq!(meta.created_by, Some("mktorrent 1.1".to_owned()));
        assert_eq!(meta.creation_date, Some(1707570148));
        assert_eq!(meta.info.mode, FileMode::Single { length: 659554304 });
        assert_eq!(meta.info.name, Some("debian-12.5.0-amd64-netinst.iso".to_owned()));
        assert_eq!(meta.info.piece_length, 262144);
        assert_eq!(meta.info.pieces.0.len(), 50320 / 20);
    }
}