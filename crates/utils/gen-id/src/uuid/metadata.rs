use uuid::Uuid;

/// Operating system type for metadata encoding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsType {
    Linux = 1,
    Windows = 2,
    MacOS = 3,
    Android = 4,
    IOS = 5,
}

impl OsType {
    /// Detect the current OS type
    #[cfg(target_os = "linux")]
    pub fn current() -> Self {
        OsType::Linux
    }

    #[cfg(target_os = "windows")]
    pub fn current() -> Self {
        OsType::Windows
    }

    #[cfg(target_os = "macos")]
    pub fn current() -> Self {
        OsType::MacOS
    }

    #[cfg(target_os = "android")]
    pub fn current() -> Self {
        OsType::Android
    }

    #[cfg(target_os = "ios")]
    pub fn current() -> Self {
        OsType::IOS
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "macos",
        target_os = "android",
        target_os = "ios",
    )))]
    pub fn current() -> Self {
        // Fallback to Linux for other Unix-like systems
        OsType::Linux
    }

    /// Encode as 3-bit value
    pub(crate) fn encode(self) -> u8 {
        self as u8
    }

    /// Decode from 3-bit value
    pub(crate) fn decode(value: u8) -> Self {
        match value & 0x07 {
            1 => OsType::Linux,
            2 => OsType::Windows,
            3 => OsType::MacOS,
            4 => OsType::Android,
            5 => OsType::IOS,
            _ => OsType::Linux, // Default fallback
        }
    }
}

/// Client metadata to embed in UUID
#[derive(Debug, Clone)]
pub struct ClientMetadata {
    /// Operating system type
    pub os_type: OsType,
    /// OS version encoded as (major, minor)
    pub os_version: (u8, u8),
    /// Hostname or machine identifier
    pub hostname: String,
    /// Optional user agent string
    pub user_agent: Option<String>,
}

impl ClientMetadata {
    /// Create metadata from current system
    #[cfg(feature = "custom-uuid")]
    #[inline]
    pub fn from_system() -> Self {
        use sysinfo::System;

        let mut sys = System::new_all();
        sys.refresh_all();

        let os_type = OsType::current();

        // Get OS version
        let os_version = if let Some(os_ver) = System::os_version() {
            Self::parse_os_version(&os_ver, os_type)
        } else {
            Self::detect_os_version()
        };

        // Get hostname
        let hostname = System::host_name().unwrap_or_else(|| "unknown".to_string());

        Self {
            os_type,
            os_version,
            hostname,
            user_agent: None,
        }
    }

    /// Create metadata from current system (fallback without sysinfo)
    #[cfg(not(feature = "custom-uuid"))]
    #[inline]
    pub fn from_system() -> Self {
        Self {
            os_type: OsType::current(),
            os_version: Self::detect_os_version(),
            hostname: Self::detect_hostname(),
            user_agent: None,
        }
    }

    /// Parse OS version string into (major, minor) tuple
    #[cfg(feature = "custom-uuid")]
    #[inline]
    fn parse_os_version(version_str: &str, _os_type: OsType) -> (u8, u8) {
        // Try to extract major.minor from version string
        let parts: Vec<&str> = version_str.split('.').collect();

        if parts.len() >= 2 {
            let major = parts[0].parse::<u8>().unwrap_or(0);
            let minor = parts[1].parse::<u8>().unwrap_or(0);

            // Clamp to 5 bits major (0-31), 4 bits minor (0-15)
            (major.min(31), minor.min(15))
        } else if let Some(first) = parts.first() {
            // Try to extract just major version
            let major = first.parse::<u8>().unwrap_or(0);
            (major.min(31), 0)
        } else {
            // Fallback to defaults
            Self::detect_os_version()
        }
    }

    /// Create metadata with custom values
    pub fn new(os_type: OsType, os_version: (u8, u8), hostname: impl Into<String>) -> Self {
        Self {
            os_type,
            os_version,
            hostname: hostname.into(),
            user_agent: None,
        }
    }

    /// Set user agent
    #[inline]
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    #[inline]
    fn detect_os_version() -> (u8, u8) {
        // Simplified version detection - could be enhanced with system calls
        #[cfg(target_os = "macos")]
        {
            // macOS version detection could use sysctl or similar
            (14, 0) // Default to recent version
        }
        #[cfg(target_os = "linux")]
        {
            (6, 0) // Default kernel version
        }
        #[cfg(target_os = "windows")]
        {
            (10, 0) // Windows 10/11
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            (0, 0)
        }
    }
}

/// Metadata extracted from a UUID
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedMetadata {
    pub timestamp_ms: u64,
    pub os_type: OsType,
    pub os_version: (u8, u8),
    pub hostname_hash: u8,
    pub extended_hash: u32,
}

/// Hash a string to 16 bits using a simple hash function
#[inline]
pub(crate) fn hash_to_u16(input: &str) -> u16 {
    let mut hash: u32 = 5381;
    for byte in input.bytes() {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(byte as u32);
    }
    (hash ^ (hash >> 16)) as u16
}

/// Hash a string to 32 bits
#[inline]
pub(crate) fn hash_to_u32(input: &str) -> u32 {
    let mut hash: u32 = 5381;
    for byte in input.bytes() {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(byte as u32);
    }
    hash
}

/// Encode OS metadata into 12 bits (3 bits type + 5 bits major + 4 bits minor)
#[inline]
pub(crate) fn encode_os_metadata(os_type: OsType, os_version: (u8, u8)) -> u16 {
    let type_bits = (os_type.encode() as u16) << 9;
    let major_bits = ((os_version.0 & 0x1F) as u16) << 4;
    let minor_bits = (os_version.1 & 0x0F) as u16;
    type_bits | major_bits | minor_bits
}

/// Decode OS metadata from 12 bits (3 bits type + 5 bits major + 4 bits minor)
#[inline]
pub(crate) fn decode_os_metadata(encoded: u16) -> (OsType, (u8, u8)) {
    let os_type = OsType::decode((encoded >> 9) as u8);
    let major = ((encoded >> 4) & 0x1F) as u8;
    let minor = (encoded & 0x0F) as u8;
    (os_type, (major, minor))
}

/// Extract metadata from a UUID v7 with embedded metadata
#[inline]
pub fn extract_metadata(uuid: &Uuid) -> Option<ExtractedMetadata> {
    let bytes = uuid.as_bytes();

    // Check if it's a v7 UUID
    if (bytes[6] >> 4) != 0x07 {
        return None;
    }

    // Extract timestamp (bytes 0-5)
    let timestamp_ms = u64::from_be_bytes([
        0, 0, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5],
    ]);

    // Extract OS metadata from bytes 6-7
    let os_encoded = ((bytes[6] as u16 & 0x0F) << 8) | (bytes[7] as u16);
    let (os_type, os_version) = decode_os_metadata(os_encoded);

    // Extract hostname hash from byte 9
    let hostname_hash = bytes[9];

    // Extract extended hash from bytes 10-13
    let mut extended_hash_bytes = [0u8; 4];
    extended_hash_bytes.copy_from_slice(&bytes[10..14]);
    let extended_hash = u32::from_be_bytes(extended_hash_bytes);

    Some(ExtractedMetadata {
        timestamp_ms,
        os_type,
        os_version,
        hostname_hash,
        extended_hash,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_type_encoding_decoding() {
        let test_cases = [
            OsType::Linux,
            OsType::Windows,
            OsType::MacOS,
            OsType::Android,
            OsType::IOS,
        ];

        for os_type in &test_cases {
            let encoded = os_type.encode();
            let decoded = OsType::decode(encoded);
            assert_eq!(*os_type, decoded);
        }
    }

    #[test]
    fn test_client_metadata_from_system() {
        let metadata = ClientMetadata::from_system();

        // OS type should match current platform
        assert_eq!(metadata.os_type, OsType::current());

        // Hostname should not be empty
        assert!(!metadata.hostname.is_empty());
    }

    #[test]
    fn test_metadata_different_hostnames() {
        use crate::uuid::{UuidGenerator, parse_uuid_with_metadata};

        let generator = UuidGenerator::v7();

        let metadata1 = ClientMetadata::new(OsType::Linux, (5, 0), "host-001");
        let metadata2 = ClientMetadata::new(OsType::Linux, (5, 0), "host-002");

        let uuid1 = generator.generate_with_metadata(&metadata1);
        let uuid2 = generator.generate_with_metadata(&metadata2);

        let (_, ext1) = parse_uuid_with_metadata(&uuid1).unwrap();
        let (_, ext2) = parse_uuid_with_metadata(&uuid2).unwrap();

        // Extended hashes should differ due to different hostnames
        let meta1 = ext1.expect("ext1 should have metadata");
        let meta2 = ext2.expect("ext2 should have metadata");
        assert_ne!(meta1.extended_hash, meta2.extended_hash);
    }

    #[test]
    fn test_encode_decode_os_metadata() {
        let test_cases = [
            (OsType::Linux, (5, 15)),
            (OsType::Windows, (10, 0)),
            (OsType::MacOS, (14, 5)),
            (OsType::Android, (13, 0)),
            (OsType::IOS, (17, 2)), // 5-bit major version supports up to 31
        ];

        for (os_type, os_version) in &test_cases {
            let encoded = encode_os_metadata(*os_type, *os_version);
            let (decoded_type, decoded_version) = decode_os_metadata(encoded);

            assert_eq!(*os_type, decoded_type);
            assert_eq!(*os_version, decoded_version);
        }
    }

    #[test]
    fn test_hash_consistency() {
        let input = "test-hostname";

        // Hash should be consistent
        let hash1 = hash_to_u16(input);
        let hash2 = hash_to_u16(input);
        assert_eq!(hash1, hash2);

        let hash1_32 = hash_to_u32(input);
        let hash2_32 = hash_to_u32(input);
        assert_eq!(hash1_32, hash2_32);
    }

    #[test]
    fn test_hash_different_inputs() {
        let hash1 = hash_to_u16("host-001");
        let hash2 = hash_to_u16("host-002");

        // Different inputs should (likely) produce different hashes
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_version_clamping() {
        // Test that versions exceeding bit limits are properly masked
        // Major: 5-bit (0-31), Minor: 4-bit (0-15)
        let test_cases = [
            ((31, 15), (31, 15)), // Max values for 5-bit major, 4-bit minor
            ((32, 0), (0, 0)),    // 32 & 0x1F = 0 (overflow)
            ((17, 2), (17, 2)),   // Valid iOS 17.2
            ((25, 16), (25, 0)),  // Minor overflow: 16 & 0x0F = 0
        ];

        for (input_version, expected_version) in &test_cases {
            let encoded = encode_os_metadata(OsType::Linux, *input_version);
            let (_, decoded_version) = decode_os_metadata(encoded);
            assert_eq!(*expected_version, decoded_version);
        }
    }
}
