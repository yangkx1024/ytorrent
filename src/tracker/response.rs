use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddrV4};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Error;
use serde_with::{DeserializeAs, SerializeAs};
use serde_with::rust::unwrap_or_skip;

use super::*;

#[derive(Deserialize, Debug)]
pub struct TrackerResponseCompat {
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        with = "unwrap_or_skip"
    )]
    pub complete: Option<u64>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        with = "unwrap_or_skip"
    )]
    pub incomplete: Option<u64>,
    pub interval: u64,
    pub peers: CompactPeers,
}

#[derive(Debug)]
pub struct CompactPeers(pub Vec<SocketAddrV4>);

impl Serialize for CompactPeers {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut bytes = Vec::with_capacity(self.0.len() * 6);
        for addr in self.0.as_slice() {
            bytes.extend_from_slice(&addr.ip().octets());
            bytes.extend_from_slice(&addr.port().to_be_bytes());
        }
        serde_with::Bytes::serialize_as(&bytes, serializer)
    }
}

impl<'de> Deserialize<'de> for CompactPeers {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: &[u8] = serde_with::Bytes::deserialize_as(deserializer)?;
        if bytes.len() % 6 != 0 {
            return Err(Error::custom(format!(
                "buffer length {} is not a multiple of {}",
                bytes.len(),
                6
            )));
        }
        let address_list = bytes
            .chunks_exact(6)
            .map(|chunk| {
                let ip_slice: &[u8; 4] = &chunk[0..4].try_into().unwrap();
                let ip = Ipv4Addr::from(*ip_slice);
                let port_slice: &[u8; 2] = &chunk[4..6].try_into().unwrap();
                let port = u16::from_be_bytes(*port_slice);
                SocketAddrV4::new(ip, port)
            })
            .collect();
        Ok(Self(address_list))
    }
}

#[derive(Deserialize, Debug)]
pub struct ScrapeResponse {
    pub files: HashMap<Sha1Digest, ScrapeFile>,
}

#[derive(Deserialize, Debug)]
pub struct ScrapeFile {
    pub complete: i64,
    pub downloaded: i64,
    pub incomplete: i64,
}
