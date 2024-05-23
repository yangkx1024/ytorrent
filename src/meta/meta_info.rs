use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use serde_with::rust::unwrap_or_skip;
use serde_with::SerializeAs;

use super::*;

pub type AnnounceList = Vec<Vec<String>>;

#[derive(Deserialize, Debug)]
pub struct MetaInfo {
    /// The URL of the tracker.
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        with = "unwrap_or_skip"
    )]
    pub announce: Option<String>,
    /// [BEP-0012](https://www.bittorrent.org/beps/bep_0012.html) extends BitTorrent to support
    /// multiple trackers
    #[serde(
        rename = "announce-list",
        skip_serializing_if = "Option::is_none",
        default,
        with = "unwrap_or_skip"
    )]
    pub announce_list: Option<AnnounceList>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        with = "unwrap_or_skip"
    )]
    pub comment: Option<String>,
    #[serde(
        rename = "created by",
        skip_serializing_if = "Option::is_none",
        default,
        with = "unwrap_or_skip"
    )]
    pub created_by: Option<String>,
    #[serde(
        rename = "creation date",
        skip_serializing_if = "Option::is_none",
        default,
        with = "unwrap_or_skip"
    )]
    pub creation_date: Option<u64>,
    pub info: Info,
    /// [BEP-0005](https://www.bittorrent.org/beps/bep_0005.html#entropy)
    /// DHT support
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        with = "unwrap_or_skip"
    )]
    pub nodes: Option<Vec<Node>>,
    #[serde(
        rename = "url-list",
        skip_serializing_if = "Option::is_none",
        default,
        with = "unwrap_or_skip"
    )]
    pub url_list: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
pub struct Info {
    /// Single or Multiple files
    #[serde(flatten)]
    pub mode: FileMode,
    /// The name key maps to a UTF-8 encoded string which is the suggested name to save the file
    /// (or directory) as. It is purely advisory.
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        with = "unwrap_or_skip"
    )]
    pub name: Option<String>,
    /// piece length maps to the number of bytes in each piece the file is split into. For the
    /// purposes of transfer, files are split into fixed-size pieces which are all the same length
    /// except for possibly the last one which may be truncated. piece length is almost always a
    /// power of two, most commonly 2 18 = 256 K (BitTorrent prior to version 3.2 uses 2 20 = 1 M
    /// as default).
    #[serde(rename = "piece length")]
    pub piece_length: u64,
    /// pieces maps to a string whose length is a multiple of 20. It is to be subdivided into
    /// strings of length 20, each of which is the SHA1 hash of the piece at the corresponding index.
    pub pieces: PieceList,
    /// [BEP-0027](https://www.bittorrent.org/beps/bep_0027.html)
    /// extends BitTorrent to support private torrents.
    /// When generating a metainfo file, users denote a torrent as private by including the
    /// key-value pair "private=1" in the "info" dict of the torrent's metainfo file
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        with = "unwrap_or_skip"
    )]
    pub private: Option<bool>,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum FileMode {
    Single { length: u64 },
    Multiple { files: Vec<FileInfo> },
}

#[derive(Debug, PartialEq)]
pub struct PieceList(
    /// SHA-1 digest
    pub Vec<Sha1Digest>,
);

impl Serialize for PieceList {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut bytes = Vec::with_capacity(self.0.len() * Sha1Digest::LENGTH);

        for piece in self.0.as_slice() {
            bytes.extend_from_slice(piece);
        }

        serde_with::Bytes::serialize_as(&bytes, serializer)
    }
}

impl<'de> Deserialize<'de> for PieceList {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = <&[u8]>::deserialize(deserializer)?;
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

#[derive(Deserialize, Debug, PartialEq)]
pub struct FileInfo {
    pub length: u64,
    pub path: Vec<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Node {
    pub host: String,
    pub port: u16,
}

impl Node {
    fn new(host: String, port: u16) -> Self {
        Node { host, port }
    }
}

impl<'de> Deserialize<'de> for Node {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (host, port) = <(String, u16)>::deserialize(deserializer)?;
        Ok(Node::new(host, port))
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;

    use super::*;

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

    trait ToBencode {
        fn to_bencode(&self) -> serde_bencode::Result<Vec<u8>>;
    }

    impl ToBencode for bool {
        fn to_bencode(&self) -> serde_bencode::Result<Vec<u8>> {
            serde_bencode::to_bytes(self)
        }
    }

    impl ToBencode for &str {
        fn to_bencode(&self) -> serde_bencode::Result<Vec<u8>> {
            serde_bencode::to_bytes(self)
        }
    }

    impl ToBencode for i32 {
        fn to_bencode(&self) -> serde_bencode::Result<Vec<u8>> {
            serde_bencode::to_bytes(self)
        }
    }

    #[test]
    fn test_se_de_piece_list() {
        let piece_list = PieceList([Sha1Digest::new(SAMPLE_SHA1_DIGEST)].into());
        let bytes = serde_bencode::to_bytes(&piece_list).unwrap();
        let piece_list: PieceList = de::from_bytes(&bytes).unwrap();
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
        let piece_list = PieceList([Sha1Digest::new(SAMPLE_SHA1_DIGEST.to_owned())].into());
        info.extend(serde_bencode::to_bytes(&piece_list).unwrap());
        info.extend(TAG_PRIVATE.to_bencode().unwrap());
        info.extend(false.to_bencode().unwrap());
        info.push(b'e');
        info
    }

    #[test]
    fn test_info_struct() {
        let info = build_info_data();
        let ret: Info = de::from_bytes(info.as_slice()).unwrap();
        assert_eq!(ret.mode, FileMode::Single { length: 1024 });
        assert_eq!(ret.name, Some(SAMPLE_NAME.into()));
        assert_eq!(ret.piece_length, 4096);
        assert_eq!(
            ret.pieces,
            PieceList([Sha1Digest::new(SAMPLE_SHA1_DIGEST.to_owned())].into())
        );
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
        println!("{:?}", std::str::from_utf8(&meta).unwrap());
        let ret: MetaInfo = de::from_bytes(meta.as_slice()).unwrap();
        assert_eq!(ret.announce, Some(SAMPLE_ANNOUNCE.into()));
        assert_eq!(
            ret.announce_list,
            Some(
                [
                    [
                        "http://bttracker.debian.org:6969/announce".into(),
                        "http://bttracker.debian.org:6969/announce".into(),
                    ]
                    .into(),
                    [
                        "http://bttracker.debian.org:6969/announce".into(),
                        "http://bttracker.debian.org:6969/announce".into(),
                    ]
                    .into()
                ]
                .into()
            ),
        );
        assert_eq!(
            ret.nodes,
            Some(
                [
                    Node::new("127.0.0.1".into(), 6881),
                    Node::new("your.router.node".into(), 4804),
                    Node::new("2001:db8:100:0:d5c8:db3f:995e:c0f7".into(), 1941),
                ]
                .into()
            )
        )
    }

    #[test]
    fn test_decode_debian_torrent() {
        let mut file = File::open("./resources/debian-12.5.0-amd64-netinst.iso.torrent").unwrap();
        let mut buffer = vec![];
        file.read_to_end(&mut buffer).expect("Failed to read file");
        let meta: MetaInfo = de::from_bytes(buffer.as_slice()).unwrap();
        assert_eq!(
            meta.announce,
            Some("http://bttracker.debian.org:6969/announce".into())
        );
        assert_eq!(meta.created_by, Some("mktorrent 1.1".into()));
        assert_eq!(meta.creation_date, Some(1707570148));
        assert_eq!(meta.info.mode, FileMode::Single { length: 659554304 });
        assert_eq!(
            meta.info.name,
            Some("debian-12.5.0-amd64-netinst.iso".into())
        );
        assert_eq!(meta.info.piece_length, 262144);
        assert_eq!(meta.info.pieces.0.len(), 50320 / 20);
    }
}
