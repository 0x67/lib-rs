use uuid::Uuid;

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
    pub fn v4() -> Self {
        Self::new(UuidVersion::V4, UuidFormat::Standard)
    }

    /// Create a UUID v7 generator with standard format
    pub fn v7() -> Self {
        Self::new(UuidVersion::V7, UuidFormat::Standard)
    }

    /// Set the output format
    pub fn with_format(mut self, format: UuidFormat) -> Self {
        self.format = format;
        self
    }

    /// Set a prefix for the generated UUIDs
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Remove the prefix
    pub fn without_prefix(mut self) -> Self {
        self.prefix = None;
        self
    }

    /// Generate a single UUID
    pub fn generate(&self) -> String {
        let uuid = match self.version {
            UuidVersion::V4 => Uuid::new_v4(),
            UuidVersion::V7 => Uuid::now_v7(),
        };

        self.format_uuid(&uuid)
    }

    /// Generate a batch of UUIDs
    pub fn generate_batch(&self, count: usize) -> Vec<String> {
        (0..count).map(|_| self.generate()).collect()
    }

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

/// Parse a UUID string using uuid-simd
#[cfg(feature = "simd")]
pub fn parse_uuid(input: &str) -> Result<Uuid, uuid_simd::Error> {
    use uuid_simd::UuidExt;

    let clean_input = clean_uuid_input(input);

    Uuid::parse(clean_input.as_bytes())
}

/// Parse a UUID string using standard parser
#[cfg(not(feature = "simd"))]
pub fn parse_uuid(input: &str) -> Result<Uuid, uuid::Error> {
    let clean_input = clean_uuid_input(input);

    Uuid::parse_str(clean_input)
}

fn clean_uuid_input(input: &str) -> &str {
    input
        .trim_start_matches("urn:uuid:")
        .trim_start_matches("uuid:")
        .trim_start_matches('{')
        .trim_end_matches('}')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v4_generation() {
        let generator = UuidGenerator::v4();
        let uuid = generator.generate();

        // Standard format should have hyphens
        assert_eq!(uuid.len(), 36);
        assert_eq!(uuid.chars().filter(|&c| c == '-').count(), 4);

        // Verify it's a valid UUID
        assert!(parse_uuid(&uuid).is_ok());
    }

    #[test]
    fn test_v7_generation() {
        let generator = UuidGenerator::v7();
        let uuid = generator.generate();

        // Standard format should have hyphens
        assert_eq!(uuid.len(), 36);
        assert_eq!(uuid.chars().filter(|&c| c == '-').count(), 4);

        // Verify it's a valid UUID
        assert!(parse_uuid(&uuid).is_ok());
    }

    #[test]
    fn test_v7_sortability() {
        let generator = UuidGenerator::v7();

        // Generate multiple UUIDs with small delays
        let mut uuids = Vec::new();
        for _ in 0..5 {
            uuids.push(generator.generate());
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        // Check that they are lexicographically sorted
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

        // Simple format should not have hyphens
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

        // Parse without prefix
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

        // All should be valid UUIDs
        for uuid in &batch {
            assert!(parse_uuid(uuid).is_ok());
        }

        // All should be unique
        let unique: std::collections::HashSet<_> = batch.iter().collect();
        assert_eq!(unique.len(), 10);
    }

    #[test]
    fn test_batch_generation_v7() {
        let generator = UuidGenerator::v7();
        let batch = generator.generate_batch(10);

        assert_eq!(batch.len(), 10);

        // All should be valid UUIDs
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

            // Check uppercase
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
    fn test_parse_uuid_standard() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let result = parse_uuid(uuid_str);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_uuid_simple() {
        let uuid_str = "550e8400e29b41d4a716446655440000";
        let result = parse_uuid(uuid_str);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_uuid_uppercase() {
        let uuid_str = "550E8400-E29B-41D4-A716-446655440000";
        let result = parse_uuid(uuid_str);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_uuid_with_urn_prefix() {
        let uuid_str = "urn:uuid:550e8400-e29b-41d4-a716-446655440000";
        let result = parse_uuid(uuid_str);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_uuid_with_braces() {
        let uuid_str = "{550e8400-e29b-41d4-a716-446655440000}";
        let result = parse_uuid(uuid_str);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid_uuid() {
        let uuid_str = "invalid-uuid-string";
        let result = parse_uuid(uuid_str);
        assert!(result.is_err());
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
}
