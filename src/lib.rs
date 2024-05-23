//! This lib provides deserialize impl for bencode.
//!
//! Example:
//!
//! ```
//! use std::collections::HashMap;
//! use serde::Deserialize;
//! use serde_with::{serde_as, Bytes};
//! use ytorrent::de;
//!
//! #[serde_as]
//! #[derive(Deserialize)]
//! struct Foo {
//!     str: String,
//!     int: i32,
//!     #[serde_as(as = "Bytes")]
//!     bytes: Vec<u8>,
//!     map: HashMap<String, String>
//! }
//! let data = b"d3:str4:demo3:inti1e5:bytes4:12343:mapd4:key16:value1ee";
//!
//! let foo: Foo = de::from_bytes(data).unwrap();
//! assert_eq!(foo.str, "demo".to_string());
//! ```
//!
//! Also provides deserialize impl for torrent file.
//!
//! Example:
//!
//! ```
//! use ytorrent::{Client, MetaInfo};
//!
//! let client = Client::new("./resources/debian-12.5.0-amd64-netinst.iso.torrent");
//! let meta: MetaInfo = client.torrent.meta_info;
//! assert_eq!(meta.announce, Some("http://bttracker.debian.org:6969/announce".into()));
//! ```
//!
pub use bencode::*;
pub use common::*;
pub use meta::*;
pub use tracker::*;

mod bencode;
mod common;
mod meta;
mod tracker;

#[cfg(test)]
mod tests {}
