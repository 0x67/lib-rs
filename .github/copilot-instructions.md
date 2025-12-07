## Rust Code Guidelines

### Working Philosophy

- Only code when you have HIGH CONFIDENCE (>80%) your suggestion is correct.
- If unsure, ask for clarification instead of guessing.
- When reviewing text, only comment on clarity issues if the text is genuinely confusing or could lead to errors.
- Don't write documentation files when finish writing code.

### Security & Safety

- Unsafe code blocks without justification
- Command injection risks (shell commands, user input)
- Path traversal vulnerabilities
- Credential exposure or hardcoded secrets
- Missing input validation on external data
- Improper error handling that could leak sensitive info

### Correctness Issues

- Logic errors that could cause panics or incorrect behavior
- Race conditions in async code
- Resource leaks (files, connections, memory)
- Off-by-one errors or boundary conditions
- Incorrect error propagation (using `unwrap()` inappropriately)
- Optional types that don’t need to be optional
- Booleans that should default to false but are set as optional
- Error context that doesn’t add useful information
- Overly defensive code with unnecessary checks
- Unnecessary comments that restate obvious code behavior

### Architecture & Patterns

- Code that violates existing patterns in the codebase
- Missing error handling (should use `anyhow::Result` for applications and `thiserror` for libraries) not `unwrap()`
- Async/await misuse or blocking operations in async contexts
- Improper trait implementations

### Error Handling Guidelines

Follow modular error design principles from [Sabrina Jewson's blog](https://sabrinajewson.org/blog/errors#guidelines-for-good-errors). Use `thiserror` to reduce boilerplate while maintaining these patterns:

**Core Principle**: Error types should be located near their unit of fallibility.

**Structure Pattern** (adapt to your use case):

```rust
// For each fallible operation, create a specific error type
#[derive(Debug, thiserror::Error)]
#[error("error reading `{path}`")]
pub struct FromFileError {
    pub path: Box<Path>,
    #[source]
    pub kind: FromFileErrorKind,
}

#[derive(Debug, thiserror::Error)]
pub enum FromFileErrorKind {
    #[error("failed to read file")]
    ReadFile(#[from] io::Error),

    #[error("failed to parse file")]
    Parse(#[from] ParseError),
}
```

**Key Guidelines**:

1. **Separate error types per operation** - Don't use a single library-wide error enum

   - Ask: Do they fail differently? Do they need different messages?
   - If yes to either, use separate types

2. **Use `.source()` chains for context** - Leverage `#[source]` or `#[from]` attributes

   - Top level: Operation context (e.g., "error reading `file.txt`")
   - Middle layers: Specific failure points (e.g., "invalid data on line 223")
   - Bottom: Root cause (e.g., "invalid digit in string")

3. **Make errors extensible** - Use `#[non_exhaustive]` on structs and relevant enum variants

   ```rust
   #[derive(Debug, thiserror::Error)]
   #[non_exhaustive]
   pub struct ParseError {
       pub line: usize,
       #[source]
       pub kind: ParseErrorKind,
   }
   ```

4. **Be specific with variant names** - Use `ReadFile(io::Error)` not `Io(io::Error)`

   - The variant name adds semantic meaning about where/why the error occurred

5. **Implement `From` carefully** - Only when semantically appropriate

   - ✅ `From<ParseError>` for variant `Parse(ParseError)`
   - ❌ `From<io::Error>` for variant `ReadFile(io::Error)` (too implicit)

6. **Follow std conventions**:

   - Error messages: lowercase, no trailing punctuation
   - No "Error" suffix in variant names (it's redundant)
   - Use `Display` for the current layer, `source()` for the cause

7. **Keep errors near their functions** - Define error types in same module as the function
   - Avoid "errors" modules (they're organizational junk drawers)
   - Place error definitions right after the relevant `impl` block

**When to create a new error type**:

- Each public function that can fail in different ways
- Operations that need distinct error messages or context
- When you need to expose what can go wrong for precise matching

**Example with thiserror**:

```rust
use thiserror::Error;

impl Blocks {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, FromFileError> {
        // Implementation
    }
}

#[derive(Debug, Error)]
#[error("error reading `{}`", path.display())]
#[non_exhaustive]
pub struct FromFileError {
    pub path: Box<Path>,
    #[source]
    pub kind: FromFileErrorKind,
}

#[derive(Debug, Error)]
pub enum FromFileErrorKind {
    #[error("failed to read file")]
    ReadFile(#[from] io::Error),

    #[error("failed to parse content")]
    Parse(#[from] ParseError),
}

#[derive(Debug, Error)]
#[error("invalid data on line {}", line + 1)]
#[non_exhaustive]
pub struct ParseError {
    pub line: usize,
    #[source]
    pub kind: ParseErrorKind,
}

#[derive(Debug, Error)]
pub enum ParseErrorKind {
    #[error("missing semicolon")]
    #[non_exhaustive]
    NoSemicolon,

    #[error("missing range separator")]
    #[non_exhaustive]
    NoDotDot,

    #[error("invalid hexadecimal integer")]
    #[non_exhaustive]
    ParseInt {
        #[source]
        source: ParseIntError,
    },
}
```

**Benefits of this approach**:

- ✅ Rich backtraces with `anyhow` or similar
- ✅ Extensible without breaking changes
- ✅ Precise error matching for callers
- ✅ Clear about what errors each function can produce
- ✅ Modular - easy to extract components
- ✅ Hides implementation details (private dependencies)

## Skip These (Low Value)

Do not comment on:

- Style/formatting (rustfmt, prettier)
- Clippy warnings
- Test failures
- Missing dependencies (npm ci covers this)
- Minor naming suggestions
- Suggestions to add comments
- Refactoring unless addressing a real bug
- Multiple issues in one comment
- Logging suggestions unless security-related
- Pedantic text accuracy unless it affects meaning
