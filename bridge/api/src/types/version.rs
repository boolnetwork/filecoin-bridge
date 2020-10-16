use std::fmt;
use serde::{Serialize, Deserialize};

/// Version provides various build-time information.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Version {
    /// User version (build version + current commit)
    pub version: String,
    /// api_version is a semver version of the rpc api exposed
    #[serde(rename = "APIVersion")]
    pub api_version: BuildVersion,
    /// Seconds
    pub block_delay: u64,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}+api{}", self.version, self.api_version)
    }
}

/// BuildVersion is the local build version, set by build system
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BuildVersion(u32);

impl fmt::Display for BuildVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (major, minor, patch) = self.semver();
        write!(f, "{}.{}.{}", major, minor, patch)
    }
}

impl From<(u8, u8, u8)> for BuildVersion {
    fn from((major, minor, patch): (u8, u8, u8)) -> Self {
        Self::new((major, minor, patch))
    }
}

impl BuildVersion {
    /// Create a new build version.
    pub fn new((major, minor, patch): (u8, u8, u8)) -> Self {
        Self(u32::from(major) << 16 | u32::from(minor) << 8 | u32::from(patch))
    }

    /// Return the version with the (major, minor, patch) format.
    pub fn semver(self) -> (u8, u8, u8) {
        (self.major(), self.minor(), self.patch())
    }

    /// Return the major version.
    pub fn major(self) -> u8 {
        (self.0 & MAJOR_ONLY_MASK >> 16) as u8
    }

    /// Return the minor version.
    pub fn minor(self) -> u8 {
        ((self.0 & MINOR_ONLY_MASK) >> 8) as u8
    }

    /// Return the patch version.
    pub fn patch(self) -> u8 {
        (self.0 & PATCH_ONLY_MASK) as u8
    }
}

const MAJOR_ONLY_MASK: u32 = 0x00ff_0000;
const MINOR_ONLY_MASK: u32 = 0x0000_ff00;
const PATCH_ONLY_MASK: u32 = 0x0000_00ff;