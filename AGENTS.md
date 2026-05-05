# Workspace Note

Shared agent instructions (lat.md, fff, rust-analyzer, code comments) are inherited from user-level global configs. This file contains only project-specific overrides.

See: `~/.github/copilot-instructions.md`, `~/.copilot/AGENTS.md`, `~/.claude/CLAUDE.md`

# Memory And Search Protocol (MANDATORY)

All agents (Conductor, subagents, standalone) MUST follow this order before planning, implementation, review, investigation, writing code, or delegating research:

1. Call `agentmemory/memory_recall` with task, file, and module keywords when available.
2. Use `lat locate` or `lat expand` for architecture and design context when `lat.md/` exists.
3. Use Semble for semantic code search: `uvx --from "semble[mcp]" semble search "query" .`.
4. Use `fff`/`fff-mcp` or fff MCP tools for exact/file search.
5. Use `rust-analyzer` for Rust definitions, references, hover, diagnostics.
6. Fall back to regular search/read tools if preferred tools are missing, fail, or lack needed capability. State fallback reason.

Fallback rule: if preferred tool is missing, fails, or lacks needed capability, use regular tools and state reason in response or handoff.

Subagents should use Semble CLI fallback because MCP tool schemas may be unavailable in nested agent context.

Conductor prompts must repeat memory/search protocol and fallback behavior for subagents.

# Unit Test Rules

**Unit tests MUST NOT connect to external services** - databases (PostgreSQL, MSSQL), APIs, or network resources.

Rules:

- **No real service connections in unit tests** - no DB connections, HTTP clients, external APIs
- **Use `#[ignore]` for integration tests** - tests requiring real PostgreSQL/MSSQL/network services must be annotated with `#[ignore]` and only run via `cargo test -- --ignored`
- **Use mockall for mocking** - prefer [mockall](https://docs.rs/mockall/latest/mockall/) crate for creating mock implementations of traits and functions
- **Localhost mock servers acceptable** - tests that bind to `127.0.0.1:0` with ephemeral ports and implement mock protocol servers in-process are acceptable
- **E2E tests are exempt** - only apply these rules when creating unit/integration tests, NOT when user explicitly asks for e2e tests
- Run unit tests using `cargo nextest` for faster feedback loops.

  When writing new tests:

1. Default to pure unit tests using test doubles/mocks
2. Add mockall to dev-dependencies if mocking is needed: `mockall = { workspace = true }`
3. Gate any DB/API tests with `#[ignore]` attribute
4. Document in test comments when `--ignored` flag is required
5. **Test behavior, not language features** - do not write tests that verify language semantics (e.g., `Option::is_some()`, type casts, serde deserialization, default trait values). Tests should verify project-specific business logic and behavior
