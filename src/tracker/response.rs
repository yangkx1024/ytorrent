use std::net::{Ipv4Addr, SocketAddrV4};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Error;
use serde_with::rust::unwrap_or_skip;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct TrackerResponseCompat {
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        with = "unwrap_or_skip"
    )]
    complete: Option<u64>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        with = "unwrap_or_skip"
    )]
    incomplete: Option<u64>,
    interval: u64,
    peers: CompactPeers,
}

#[derive(Debug)]
pub(crate) struct CompactPeers(Vec<SocketAddrV4>);

impl Serialize for CompactPeers {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut bytes = Vec::with_capacity(self.0.len() * 6);
        for addr in &self.0 {
            bytes.extend_from_slice(&addr.ip().octets());
            bytes.extend_from_slice(&addr.port().to_be_bytes());
        }
        serde_bytes::Bytes::new(bytes.as_slice()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CompactPeers {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let bytes = serde_bytes::ByteBuf::deserialize(deserializer)?.into_vec();
        if bytes.len() % 6 != 0 {
            return Err(Error::custom(format!(
                "buffer length {} is not a multiple of {}",
                bytes.len(),
                6
            )));
        }
        let address_list = bytes.chunks_exact(6)
            .map(|chunk| {
                let ip_slice: &[u8; 4] = &chunk[0..4].try_into().unwrap();
                let ip = Ipv4Addr::from(*ip_slice);
                let port_slice: &[u8; 2] = &chunk[4..6].try_into().unwrap();
                let port = u16::from_be_bytes(*port_slice);
                SocketAddrV4::new(ip, port)
            }).collect();
        Ok(Self(address_list))
    }
}