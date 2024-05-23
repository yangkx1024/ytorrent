use std::path::Path;

use rand::random;
use url::form_urlencoded::byte_serialize;

use super::*;

pub struct Client {
    pub torrent: Torrent,
}

impl Client {
    /// Construct a [Client] from a torrent file
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            torrent: Torrent::parse(path),
        }
    }

    pub async fn connect_announce(&self) -> Result<TrackerResponseCompat> {
        let peer_id: [u8; 20] = random();
        let info_hash_query: String = byte_serialize(self.torrent.info_hash.as_ref()).collect();
        let peer_id_query: String = byte_serialize(&peer_id).collect();
        let http_url = format!(
            "{}?info_hash={}&peer_id={}&compact=1",
            self.torrent.meta_info.announce.as_ref().unwrap(),
            info_hash_query,
            peer_id_query
        );
        if cfg!(debug) || cfg!(test) {
            println!("url: {}", http_url);
        }
        let ret = reqwest::get(http_url).await?;
        let bytes = ret.bytes().await?;
        if cfg!(debug) || cfg!(test) {
            println!("response {:?}", bytes);
        }
        let response: TrackerResponseCompat = de::from_bytes(&bytes)?;
        Ok(response)
    }

    pub async fn connect_scrape(&self) -> Result<ScrapeFile> {
        let announce_url = self.torrent.meta_info.announce.as_ref().unwrap();
        let scrape_url = announce_url.replacen("announce", "scrape", 1);
        let info_hash_query: String = byte_serialize(self.torrent.info_hash.as_ref()).collect();

        let http_url = format!("{}?info_hash={}", scrape_url, info_hash_query);
        if cfg!(debug) || cfg!(test) {
            println!("url: {}", http_url);
        }
        let ret = reqwest::get(http_url).await?;
        let bytes = ret.bytes().await?;
        if cfg!(debug) || cfg!(test) {
            println!("response {:?}", bytes);
        }
        let mut response: ScrapeResponse = de::from_bytes(bytes.as_ref())?;
        response
            .files
            .remove(&self.torrent.info_hash)
            .ok_or(Error::Request("Failed to fetch file info".to_string()))
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
        assert!(resp.is_ok());
    }

    #[tokio::test]
    async fn test_connect_scrape() {
        let client = Client::new("./resources/debian-12.5.0-amd64-netinst.iso.torrent");
        let resp = client.connect_scrape().await;
        println!("{:?}", resp);
        assert!(resp.is_ok());
    }
}
