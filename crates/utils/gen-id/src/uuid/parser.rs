use uuid::Uuid;

#[cfg(feature = "custom-uuid")]
use super::metadata::{ExtractedMetadata, extract_metadata};

/// Error type for UUID parsing
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[cfg(feature = "simd")]
    #[error("SIMD UUID parse error: {0:?}")]
    Simd(#[from] uuid_simd::Error),

    #[cfg(not(feature = "simd"))]
    #[error("UUID parse error: {0}")]
    Standard(#[from] uuid::Error),
}

/// Parse a UUID string and extract embedded metadata if present
///
/// # Availability
/// This function is only available when the `custom-uuid` feature is enabled.
#[inline]
#[cfg(feature = "custom-uuid")]
pub fn parse_uuid_with_metadata(
    input: &str,
) -> Result<(Uuid, Option<ExtractedMetadata>), ParseError> {
    let clean_input = clean_uuid_input(input);

    #[cfg(feature = "simd")]
    let uuid = {
        use uuid_simd::UuidExt;
        Uuid::parse(clean_input.as_bytes()).map_err(ParseError::Simd)?
    };

    #[cfg(not(feature = "simd"))]
    let uuid = Uuid::parse_str(clean_input).map_err(ParseError::Standard)?;

    let metadata = extract_metadata(&uuid);
    Ok((uuid, metadata))
}

/// Parse a UUID string using uuid-simd (legacy function, kept for backwards compatibility)
#[inline]
#[cfg(feature = "simd")]
pub fn parse_uuid(input: &str) -> Result<Uuid, uuid_simd::Error> {
    use uuid_simd::UuidExt;

    let clean_input = clean_uuid_input(input);

    Uuid::parse(clean_input.as_bytes())
}

/// Parse a UUID string using standard parser (legacy function, kept for backwards compatibility)
#[inline]
#[cfg(not(feature = "simd"))]
pub fn parse_uuid(input: &str) -> Result<Uuid, uuid::Error> {
    let clean_input = clean_uuid_input(input);

    Uuid::parse_str(clean_input)
}

#[inline]
pub fn clean_uuid_input(input: &str) -> &str {
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
    #[cfg(feature = "custom-uuid")]
    fn test_extract_metadata_from_standard_v7() {
        use crate::uuid::UuidGenerator;

        let generator = UuidGenerator::v7();
        let uuid = generator.generate();

        let (_, extracted) = parse_uuid_with_metadata(&uuid).unwrap();
        assert!(extracted.is_some(), "v7 UUID should have metadata");
    }

    #[test]
    #[cfg(feature = "custom-uuid")]
    fn test_extract_metadata_from_v4() {
        use crate::uuid::UuidGenerator;

        let generator = UuidGenerator::v4();
        let uuid = generator.generate();

        let (_, extracted) = parse_uuid_with_metadata(&uuid).unwrap();
        assert!(extracted.is_none(), "v4 UUID should not have metadata");
    }
}
