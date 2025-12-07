/// Default length for generated NanoIDs
pub const DEFAULT_LENGTH: usize = 12;

/// Alphanumeric alphabet including uppercase and lowercase letters and numbers
const ALPHABET: [char; 62] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I',
    'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b',
    'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u',
    'v', 'w', 'x', 'y', 'z',
];

/// A NanoID generator with customizable length and optional prefix support
#[derive(Debug, Clone, Copy)]
pub struct NanoIdGenerator;

impl Default for NanoIdGenerator {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl NanoIdGenerator {
    /// Creates a new NanoID generator
    #[inline]
    pub const fn new() -> Self {
        Self
    }

    /// Generates a single NanoID
    ///
    /// # Arguments
    ///
    /// * `prefix` - Optional prefix to prepend to the NanoID
    /// * `length` - Optional custom length (defaults to 12 if None)
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_id::NanoIdGenerator;
    ///
    /// let generator = NanoIdGenerator::new();
    /// let id = generator.generate(None, None);
    /// assert_eq!(id.len(), 12); // default length
    ///
    /// let id_with_prefix = generator.generate(Some("user_"), None);
    /// assert!(id_with_prefix.starts_with("user_"));
    ///
    /// let id_custom_length = generator.generate(None, Some(16));
    /// assert_eq!(id_custom_length.len(), 16);
    ///
    /// let id_both = generator.generate(Some("item_"), Some(8));
    /// assert!(id_both.starts_with("item_"));
    /// assert_eq!(id_both.len(), 5 + 8); // "item_" + 8
    /// ```
    #[inline]
    pub fn generate(&self, prefix: Option<&str>, length: Option<usize>) -> String {
        let len = length.unwrap_or(DEFAULT_LENGTH);
        let nanoid = nanoid::format(nanoid::rngs::default, &ALPHABET, len);

        match prefix {
            Some(p) => format!("{}{}", p, nanoid),
            None => nanoid,
        }
    }

    /// Generates a batch of NanoIDs
    ///
    /// # Arguments
    ///
    /// * `count` - The number of NanoIDs to generate
    /// * `prefix` - Optional prefix to prepend to each NanoID
    /// * `length` - Optional custom length (defaults to 12 if None)
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_id::NanoIdGenerator;
    ///
    /// let generator = NanoIdGenerator::new();
    /// let ids = generator.generate_batch(5, None, None);
    /// assert_eq!(ids.len(), 5);
    ///
    /// let ids_with_prefix = generator.generate_batch(3, Some("order_"), None);
    /// assert_eq!(ids_with_prefix.len(), 3);
    /// assert!(ids_with_prefix[0].starts_with("order_"));
    ///
    /// let ids_custom_length = generator.generate_batch(3, None, Some(16));
    /// assert_eq!(ids_custom_length[0].len(), 16);
    /// ```
    #[inline]
    pub fn generate_batch(
        &self,
        count: usize,
        prefix: Option<&str>,
        length: Option<usize>,
    ) -> Vec<String> {
        (0..count).map(|_| self.generate(prefix, length)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_default_length() {
        let generator = NanoIdGenerator::new();
        let id = generator.generate(None, None);
        assert_eq!(id.len(), DEFAULT_LENGTH);
    }

    #[test]
    fn test_custom_length() {
        let generator = NanoIdGenerator::new();
        let id = generator.generate(None, Some(20));
        assert_eq!(id.len(), 20);
    }

    #[test]
    fn test_with_prefix() {
        let generator = NanoIdGenerator::new();
        let id = generator.generate(Some("test_"), None);
        assert!(id.starts_with("test_"));
        assert_eq!(id.len(), 5 + DEFAULT_LENGTH); // prefix + default length
    }

    #[test]
    fn test_alphanumeric_characters() {
        let generator = NanoIdGenerator::new();
        let id = generator.generate(None, None);

        // Check that all characters are alphanumeric
        for ch in id.chars() {
            assert!(ch.is_ascii_alphanumeric());
        }
    }

    #[test]
    fn test_batch_generation() {
        let generator = NanoIdGenerator::new();
        let batch_size = 10;
        let ids = generator.generate_batch(batch_size, None, None);

        assert_eq!(ids.len(), batch_size);

        // Check that all IDs have the correct length
        for id in &ids {
            assert_eq!(id.len(), DEFAULT_LENGTH);
        }
    }

    #[test]
    fn test_batch_uniqueness() {
        let generator = NanoIdGenerator::new();
        let ids = generator.generate_batch(100, None, None);

        // Convert to HashSet to check uniqueness
        let unique_ids: HashSet<_> = ids.iter().collect();
        assert_eq!(
            unique_ids.len(),
            ids.len(),
            "Generated IDs should be unique"
        );
    }

    #[test]
    fn test_batch_with_prefix() {
        let generator = NanoIdGenerator::new();
        let ids = generator.generate_batch(5, Some("item_"), Some(8));

        assert_eq!(ids.len(), 5);
        for id in ids {
            assert!(id.starts_with("item_"));
            assert_eq!(id.len(), 5 + 8); // "item_" + 8 characters
        }
    }

    #[test]
    fn test_randomness() {
        let generator = NanoIdGenerator::new();
        let id1 = generator.generate(None, None);
        let id2 = generator.generate(None, None);

        // With high probability, two generated IDs should be different
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_runtime_prefix_and_length() {
        let generator = NanoIdGenerator::new();

        // Same generator, different prefixes
        let user_id = generator.generate(Some("user_"), None);
        let order_id = generator.generate(Some("order_"), None);
        assert!(user_id.starts_with("user_"));
        assert!(order_id.starts_with("order_"));

        // Same generator, different lengths
        let short_id = generator.generate(None, Some(8));
        let long_id = generator.generate(None, Some(20));
        assert_eq!(short_id.len(), 8);
        assert_eq!(long_id.len(), 20);

        // Combine both
        let custom_id = generator.generate(Some("item_"), Some(16));
        assert!(custom_id.starts_with("item_"));
        assert_eq!(custom_id.len(), 5 + 16);
    }
}
