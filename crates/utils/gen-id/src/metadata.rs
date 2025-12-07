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

    /// Encode as 4-bit value
    pub(crate) fn encode(self) -> u8 {
        self as u8
    }

    /// Decode from 4-bit value
    pub(crate) fn decode(value: u8) -> Self {
        match value & 0x0F {
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
    fn parse_os_version(version_str: &str, _os_type: OsType) -> (u8, u8) {
        // Try to extract major.minor from version string
        let parts: Vec<&str> = version_str.split('.').collect();

        if parts.len() >= 2 {
            let major = parts[0].parse::<u8>().unwrap_or(0);
            let minor = parts[1].parse::<u8>().unwrap_or(0);

            // Clamp to 4 bits each (0-15)
            (major.min(15), minor.min(15))
        } else if let Some(first) = parts.first() {
            // Try to extract just major version
            let major = first.parse::<u8>().unwrap_or(0);
            (major.min(15), 0)
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
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

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
pub(crate) fn hash_to_u16(input: &str) -> u16 {
    let mut hash: u32 = 5381;
    for byte in input.bytes() {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(byte as u32);
    }
    (hash ^ (hash >> 16)) as u16
}

/// Hash a string to 32 bits
pub(crate) fn hash_to_u32(input: &str) -> u32 {
    let mut hash: u32 = 5381;
    for byte in input.bytes() {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(byte as u32);
    }
    hash
}

/// Encode OS metadata into 12 bits (4 bits type + 8 bits version)
pub(crate) fn encode_os_metadata(os_type: OsType, os_version: (u8, u8)) -> u16 {
    let type_bits = (os_type.encode() as u16) << 8;
    let version_bits = ((os_version.0 & 0x0F) as u16) << 4 | ((os_version.1 & 0x0F) as u16);
    type_bits | version_bits
}

/// Decode OS metadata from 12 bits
pub(crate) fn decode_os_metadata(encoded: u16) -> (OsType, (u8, u8)) {
    let os_type = OsType::decode((encoded >> 8) as u8);
    let major = ((encoded >> 4) & 0x0F) as u8;
    let minor = (encoded & 0x0F) as u8;
    (os_type, (major, minor))
}

/// Extract metadata from a UUID v7 with embedded metadata
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
