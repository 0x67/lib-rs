#[cfg(feature = "nanoid")]
mod nanoid;
mod uuid;

// Re-export UUID types
pub use uuid::{ParseError, UuidFormat, UuidGenerator, UuidVersion, parse_uuid};

// Re-export NanoID types
#[cfg(feature = "nanoid")]
pub use nanoid::NanoIdGenerator;

// Re-export metadata types when feature is enabled
#[cfg(feature = "custom-uuid")]
pub use uuid::{
    ClientMetadata, ExtractedMetadata, OsType, extract_metadata, parse_uuid_with_metadata,
};
