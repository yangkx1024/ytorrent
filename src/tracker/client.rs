use std::path::Path;

use rand::random;
use url::form_urlencoded::byte_serialize;

use super::*;

pub(crate) struct Client {
    torrent: Torrent,
}

impl Client {
    fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            torrent: Torrent::parse(path)
        }
    }

    pub(crate) async fn connect_announce(&self) -> Result<TrackerResponseCompat> {
        let peer_id: [u8; 20] = random();
        let info_hash_query: String = byte_serialize(self.torrent.info_hash.as_ref()).collect();
        let peer_id_query: String = byte_serialize(&peer_id).collect();
        let url = format!(
            "{}?info_hash={}&peer_id={}&uploaded=0&downloaded=0&left=659554304&compact=1",
            self.torrent.meta_info.announce.as_ref().unwrap(),
            info_hash_query,
            peer_id_query
        );
        let ret = reqwest::get(url).await?;
        let bytes = ret.bytes().await?;
        let response: TrackerResponseCompat = bendy::serde::from_bytes(&bytes)?;
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use crate::tracker::client::Client;

    #[tokio::test]
    async fn test_connect_tracker() {
        let client = Client::new("./resources/debian-12.5.0-amd64-netinst.iso.torrent");
        let resp = client.connect_announce().await;
        println!("{:?}", resp);
        assert!(resp.is_ok())
    }
}