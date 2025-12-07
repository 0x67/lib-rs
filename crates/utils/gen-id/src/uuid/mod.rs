mod generator;
mod parser;

#[cfg(feature = "custom-uuid")]
mod metadata;

pub use generator::{UuidFormat, UuidGenerator, UuidVersion};
pub use parser::{ParseError, parse_uuid};

#[cfg(feature = "custom-uuid")]
pub use parser::parse_uuid_with_metadata;

#[cfg(feature = "custom-uuid")]
pub use metadata::{ClientMetadata, ExtractedMetadata, OsType, extract_metadata};
