use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

use sha1_smol::Sha1;

#[derive(Debug, PartialEq)]
pub(crate) struct Sha1Digest([u8; Self::LENGTH]);

impl Sha1Digest {
    pub(crate) const LENGTH: usize = 20;

    pub(crate) fn new(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    pub(crate) fn from_data(data: impl AsRef<[u8]>) -> Self {
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