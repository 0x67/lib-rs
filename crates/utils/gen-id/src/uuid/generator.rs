use uuid::Uuid;

#[cfg(feature = "custom-uuid")]
use super::metadata::{ClientMetadata, encode_os_metadata, hash_to_u16, hash_to_u32};

/// Format for UUID output
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UuidFormat {
    /// Standard format with hyphens: 550e8400-e29b-41d4-a716-446655440000
    Standard,
    /// Simple format without hyphens: 550e8400e29b41d4a716446655440000
    Simple,
    /// Standard format with hyphens, uppercase: 550E8400-E29B-41D4-A716-446655440000
    StandardUppercase,
    /// Simple format without hyphens, uppercase: 550E8400E29B41D4A716446655440000
    SimpleUppercase,
}

/// UUID version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UuidVersion {
    /// Random UUID (version 4)
    V4,
    /// Timestamp-based sortable UUID (version 7)
    V7,
}

/// UUID generator with various formatting options
#[derive(Debug, Clone)]
pub struct UuidGenerator {
    version: UuidVersion,
    format: UuidFormat,
    prefix: Option<String>,
}

impl UuidGenerator {
    /// Create a new UUID generator with specified version and format
    pub fn new(version: UuidVersion, format: UuidFormat) -> Self {
        Self {
            version,
            format,
            prefix: None,
        }
    }

    /// Create a UUID v4 generator with standard format
    #[inline]
    pub fn v4() -> Self {
        Self::new(UuidVersion::V4, UuidFormat::Standard)
    }

    /// Create a UUID v7 generator with standard format
    #[inline]
    pub fn v7() -> Self {
        Self::new(UuidVersion::V7, UuidFormat::Standard)
    }

    /// Set the output format
    #[inline]
    pub fn with_format(mut self, format: UuidFormat) -> Self {
        self.format = format;
        self
    }

    /// Set a prefix for the generated UUIDs
    #[inline]
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Remove the prefix
    #[inline]
    pub fn without_prefix(mut self) -> Self {
        self.prefix = None;
        self
    }

    /// Generate a single UUID
    #[inline]
    pub fn generate(&self) -> String {
        let uuid = match self.version {
            UuidVersion::V4 => Uuid::new_v4(),
            UuidVersion::V7 => Uuid::now_v7(),
        };

        self.format_uuid(&uuid)
    }

    /// Generate a UUID v7 with embedded client metadata
    ///
    /// This embeds OS type, OS version, hostname hash, and user agent hash
    /// into the UUID while maintaining timestamp-based sortability.
    ///
    /// # Structure
    /// - Bytes 0-5: Timestamp (milliseconds) - preserved for sorting
    /// - Byte 6: Version (0x7X) where X contains 4 bits of OS type
    /// - Byte 7: OS version (4 bits major, 4 bits minor)
    /// - Byte 8: Variant bits (preserved)
    /// - Byte 9: Hostname hash (8 bits)
    /// - Bytes 10-13: Extended hash (user agent + hostname)
    /// - Bytes 14-15: Random bits for collision resistance
    ///
    /// # Availability
    /// This method is only available when the `custom-uuid` feature is enabled.
    #[inline]
    #[cfg(feature = "custom-uuid")]
    pub fn generate_with_metadata(&self, metadata: &ClientMetadata) -> String {
        // Start with a v7 UUID to get the timestamp
        let uuid = Uuid::now_v7();
        let mut bytes = *uuid.as_bytes();

        // Encode OS metadata (4 bits type + 8 bits version)
        let os_encoded = encode_os_metadata(metadata.os_type, metadata.os_version);

        // Inject OS type into byte 6 (preserve version bits 0x7X)
        bytes[6] = 0x70 | ((os_encoded >> 8) as u8 & 0x0F);

        // Inject OS version into byte 7
        bytes[7] = os_encoded as u8;

        // Byte 8 is preserved for variant bits (already set by Uuid::now_v7)

        // Inject hostname hash into byte 9
        let hostname_hash = hash_to_u16(&metadata.hostname);
        bytes[9] = (hostname_hash & 0xFF) as u8;

        // Create extended hash from hostname + user agent
        let extended_input = match &metadata.user_agent {
            Some(ua) => format!("{}{}", metadata.hostname, ua),
            None => metadata.hostname.clone(),
        };
        let extended_hash = hash_to_u32(&extended_input);
        bytes[10..14].copy_from_slice(&extended_hash.to_be_bytes());

        // Bytes 14-15 remain random from the original UUID v7 for collision resistance

        let custom_uuid = Uuid::from_bytes(bytes);
        self.format_uuid(&custom_uuid)
    }

    /// Generate a batch of UUIDs with metadata
    ///
    /// # Availability
    /// This method is only available when the `custom-uuid` feature is enabled.
    #[inline]
    #[cfg(feature = "custom-uuid")]
    pub fn generate_batch_with_metadata(
        &self,
        count: usize,
        metadata: &ClientMetadata,
    ) -> Vec<String> {
        (0..count)
            .map(|_| self.generate_with_metadata(metadata))
            .collect()
    }

    /// Generate a batch of UUIDs
    #[inline]
    pub fn generate_batch(&self, count: usize) -> Vec<String> {
        (0..count).map(|_| self.generate()).collect()
    }

    #[inline]
    fn format_uuid(&self, uuid: &Uuid) -> String {
        let formatted = match self.format {
            UuidFormat::Standard => uuid.hyphenated().to_string(),
            UuidFormat::Simple => uuid.simple().to_string(),
            UuidFormat::StandardUppercase => uuid.hyphenated().to_string().to_uppercase(),
            UuidFormat::SimpleUppercase => uuid.simple().to_string().to_uppercase(),
        };

        match &self.prefix {
            Some(prefix) => format!("{}{}", prefix, formatted),
            None => formatted,
        }
    }
}

impl Default for UuidGenerator {
    fn default() -> Self {
        Self::v4()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uuid::parse_uuid;

    #[test]
    fn test_v4_generation() {
        let generator = UuidGenerator::v4();
        let uuid = generator.generate();

        assert_eq!(uuid.len(), 36);
        assert_eq!(uuid.chars().filter(|&c| c == '-').count(), 4);
        assert!(parse_uuid(&uuid).is_ok());
    }

    #[test]
    fn test_v7_generation() {
        let generator = UuidGenerator::v7();
        let uuid = generator.generate();

        assert_eq!(uuid.len(), 36);
        assert_eq!(uuid.chars().filter(|&c| c == '-').count(), 4);
        assert!(parse_uuid(&uuid).is_ok());
    }

    #[test]
    fn test_v7_sortability() {
        let generator = UuidGenerator::v7();
        let mut uuids = Vec::new();

        for _ in 0..5 {
            uuids.push(generator.generate());
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        let mut sorted = uuids.clone();
        sorted.sort();
        assert_eq!(
            uuids, sorted,
            "UUID v7 should be sortable lexicographically"
        );
    }

    #[test]
    fn test_simple_format() {
        let generator = UuidGenerator::v4().with_format(UuidFormat::Simple);
        let uuid = generator.generate();

        assert_eq!(uuid.len(), 32);
        assert!(!uuid.contains('-'));
        assert!(uuid.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_standard_uppercase_format() {
        let generator = UuidGenerator::v4().with_format(UuidFormat::StandardUppercase);
        let uuid = generator.generate();

        assert_eq!(uuid.len(), 36);
        assert_eq!(uuid.chars().filter(|&c| c == '-').count(), 4);
        assert!(
            uuid.chars()
                .filter(|c| c.is_alphabetic())
                .all(|c| c.is_uppercase())
        );
    }

    #[test]
    fn test_simple_uppercase_format() {
        let generator = UuidGenerator::v4().with_format(UuidFormat::SimpleUppercase);
        let uuid = generator.generate();

        assert_eq!(uuid.len(), 32);
        assert!(!uuid.contains('-'));
        assert!(
            uuid.chars()
                .filter(|c| c.is_alphabetic())
                .all(|c| c.is_uppercase())
        );
    }

    #[test]
    fn test_with_prefix() {
        let generator = UuidGenerator::v4().with_prefix("user_");
        let uuid = generator.generate();

        assert!(uuid.starts_with("user_"));
        assert_eq!(uuid.len(), 41); // "user_" (5) + standard UUID (36)

        let uuid_part = &uuid[5..];
        assert!(parse_uuid(uuid_part).is_ok());
    }

    #[test]
    fn test_with_prefix_simple_format() {
        let generator = UuidGenerator::v4()
            .with_format(UuidFormat::Simple)
            .with_prefix("id_");
        let uuid = generator.generate();

        assert!(uuid.starts_with("id_"));
        assert_eq!(uuid.len(), 35); // "id_" (3) + simple UUID (32)
    }

    #[test]
    fn test_without_prefix() {
        let generator = UuidGenerator::v4().with_prefix("test_").without_prefix();
        let uuid = generator.generate();

        assert!(!uuid.starts_with("test_"));
        assert_eq!(uuid.len(), 36);
    }

    #[test]
    fn test_batch_generation() {
        let generator = UuidGenerator::v4();
        let batch = generator.generate_batch(10);

        assert_eq!(batch.len(), 10);

        for uuid in &batch {
            assert!(parse_uuid(uuid).is_ok());
        }

        let unique: std::collections::HashSet<_> = batch.iter().collect();
        assert_eq!(unique.len(), 10);
    }

    #[test]
    fn test_batch_generation_v7() {
        let generator = UuidGenerator::v7();
        let batch = generator.generate_batch(10);

        assert_eq!(batch.len(), 10);

        for uuid in &batch {
            assert!(parse_uuid(uuid).is_ok());
        }
    }

    #[test]
    fn test_batch_with_prefix_and_format() {
        let generator = UuidGenerator::v7()
            .with_format(UuidFormat::SimpleUppercase)
            .with_prefix("ORDER_");
        let batch = generator.generate_batch(5);

        assert_eq!(batch.len(), 5);

        for uuid in &batch {
            assert!(uuid.starts_with("ORDER_"));
            assert_eq!(uuid.len(), 38); // "ORDER_" (6) + simple UUID (32)

            let uuid_part = &uuid[6..];
            assert!(
                uuid_part
                    .chars()
                    .filter(|c| c.is_alphabetic())
                    .all(|c| c.is_uppercase())
            );
        }
    }

    #[test]
    fn test_default_generator() {
        let generator = UuidGenerator::default();
        let uuid = generator.generate();

        assert_eq!(uuid.len(), 36);
        assert!(parse_uuid(&uuid).is_ok());
    }

    #[test]
    fn test_all_format_combinations() {
        let formats = [
            UuidFormat::Standard,
            UuidFormat::Simple,
            UuidFormat::StandardUppercase,
            UuidFormat::SimpleUppercase,
        ];

        let versions = [UuidVersion::V4, UuidVersion::V7];

        for version in &versions {
            for format in &formats {
                let generator = UuidGenerator::new(*version, *format);
                let uuid = generator.generate();

                match format {
                    UuidFormat::Standard => {
                        assert_eq!(uuid.len(), 36);
                        assert_eq!(uuid.chars().filter(|&c| c == '-').count(), 4);
                    }
                    UuidFormat::Simple => {
                        assert_eq!(uuid.len(), 32);
                        assert!(!uuid.contains('-'));
                    }
                    UuidFormat::StandardUppercase => {
                        assert_eq!(uuid.len(), 36);
                        assert!(
                            uuid.chars()
                                .filter(|c| c.is_alphabetic())
                                .all(|c| c.is_uppercase())
                        );
                    }
                    UuidFormat::SimpleUppercase => {
                        assert_eq!(uuid.len(), 32);
                        assert!(
                            uuid.chars()
                                .filter(|c| c.is_alphabetic())
                                .all(|c| c.is_uppercase())
                        );
                    }
                }
            }
        }
    }

    #[test]
    #[cfg(feature = "custom-uuid")]
    fn test_metadata_generation() {
        use crate::uuid::{ClientMetadata, OsType, parse_uuid_with_metadata};

        let metadata = ClientMetadata::new(OsType::MacOS, (14, 5), "test-machine");
        let generator = UuidGenerator::v7();

        let uuid = generator.generate_with_metadata(&metadata);

        assert_eq!(uuid.len(), 36);
        assert!(parse_uuid(&uuid).is_ok());

        let (_, extracted) = parse_uuid_with_metadata(&uuid).unwrap();
        assert!(extracted.is_some());

        let extracted: crate::uuid::ExtractedMetadata = extracted.unwrap();
        assert_eq!(extracted.os_type, OsType::MacOS);
        assert_eq!(extracted.os_version, (14, 5));
    }

    #[test]
    #[cfg(feature = "custom-uuid")]
    fn test_metadata_with_prefix() {
        use crate::uuid::{ClientMetadata, OsType, parse_uuid_with_metadata};

        let metadata = ClientMetadata::new(OsType::Linux, (6, 1), "server-01");
        let generator = UuidGenerator::v7().with_prefix("trade_");

        let uuid = generator.generate_with_metadata(&metadata);

        assert!(uuid.starts_with("trade_"));
        assert_eq!(uuid.len(), 42); // "trade_" (6) + UUID (36)

        let uuid_part = &uuid[6..];
        let (_, extracted) = parse_uuid_with_metadata(uuid_part).unwrap();
        assert!(extracted.is_some());

        let extracted: crate::uuid::ExtractedMetadata = extracted.unwrap();
        assert_eq!(extracted.os_type, OsType::Linux);
        assert_eq!(extracted.os_version, (6, 1));
    }

    #[test]
    #[cfg(feature = "custom-uuid")]
    fn test_metadata_batch_generation() {
        use crate::uuid::{ClientMetadata, OsType, parse_uuid_with_metadata};

        let metadata = ClientMetadata::new(OsType::Windows, (10, 0), "workstation");
        let generator = UuidGenerator::v7();

        let batch = generator.generate_batch_with_metadata(5, &metadata);

        assert_eq!(batch.len(), 5);

        for uuid in &batch {
            assert!(parse_uuid(uuid).is_ok());

            let (_, extracted) = parse_uuid_with_metadata(uuid).unwrap();
            assert!(extracted.is_some());

            let extracted: crate::uuid::ExtractedMetadata = extracted.unwrap();
            assert_eq!(extracted.os_type, OsType::Windows);
            assert_eq!(extracted.os_version, (10, 0));
        }

        let unique: std::collections::HashSet<_> = batch.iter().collect();
        assert_eq!(unique.len(), 5);
    }

    #[test]
    #[cfg(feature = "custom-uuid")]
    fn test_metadata_sortability() {
        use crate::uuid::{ClientMetadata, OsType};

        let metadata = ClientMetadata::new(OsType::MacOS, (14, 0), "test");
        let generator = UuidGenerator::v7();

        let mut uuids = Vec::new();
        for _ in 0..5 {
            uuids.push(generator.generate_with_metadata(&metadata));
            std::thread::sleep(std::time::Duration::from_millis(2));
        }

        let mut sorted = uuids.clone();
        sorted.sort();
        assert_eq!(uuids, sorted, "Metadata UUIDs should maintain sortability");
    }

    #[test]
    #[cfg(feature = "custom-uuid")]
    fn test_metadata_with_user_agent() {
        use crate::uuid::{ClientMetadata, OsType};

        let metadata = ClientMetadata::new(OsType::Linux, (5, 15), "dev-machine")
            .with_user_agent("TradingApp/1.0");
        let generator = UuidGenerator::v7();

        let uuid1 = generator.generate_with_metadata(&metadata);
        let uuid2 = generator.generate_with_metadata(&metadata);

        assert!(parse_uuid(&uuid1).is_ok());
        assert!(parse_uuid(&uuid2).is_ok());
        assert_ne!(uuid1, uuid2);
    }
}
