use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::ops::Deref;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::DeserializeAs;
use sha1_smol::Sha1;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Sha1Digest(pub [u8; Self::LENGTH]);

impl Sha1Digest {
    pub const LENGTH: usize = 20;

    pub(super) fn new(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    pub(super) fn digest(data: impl AsRef<[u8]>) -> Self {
        Sha1::from(data).digest().into()
    }
}

impl From<sha1_smol::Digest> for Sha1Digest {
    fn from(digest: sha1_smol::Digest) -> Self {
        Self(digest.bytes())
    }
}

impl Deref for Sha1Digest {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Sha1Digest {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{byte:02x}")?;
        }

        Ok(())
    }
}

impl<'de> Deserialize<'de> for Sha1Digest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = serde_with::Bytes::deserialize_as(deserializer)?;
        Ok(Sha1Digest::new(bytes))
    }
}

impl Serialize for Sha1Digest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}
