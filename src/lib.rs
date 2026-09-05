//! # Tokenectomy 🕵️‍♂️
//!
//! **Autonomous Machine-to-Machine (M2M) Model Context Protocol (MCP) server and context surgery engine for AI coding agents.**
//!
//! Designed specifically for autonomous coding assistants (**Claude Desktop**, **Cursor**, **Cline**, **Roo Code**, **Windsurf**, and **Google Antigravity**),
//! Tokenectomy operates silently as a background sidecar. It intercepts noisy framework errors, strips 90%+ of internal runtime frames
//! (`node_modules`, `site-packages`, `.cargo/registry`), auto-redacts sensitive credentials (JWTs, database URLs, API keys),
//! and validates code syntax via Tree-sitter AST validation before prompt transmission.
//!
//! ## Key Capabilities for AI Coding Agents
//! - **Log Surgery**: Reduces massive 38K token error dumps down to <2K tokens, preserving only user-written code.
//! - **Secret Redaction**: Employs compiled, ReDoS-safe linear-time regexes (`LazyLock`) to strip credentials locally before cloud transmission.
//! - **Local Response Cache**: SHA-256 error fingerprinting with a 24-hour local TTL so identical CI/CD errors cost $0.00 in LLM API calls.
//! - **M2M MCP Stdio Protocol**: Full JSON-RPC 2.0 stdio interface exposing `get_error_context`, `search_stack_overflow`, and `apply_code_patch`.
//!
//! ## Quick Library Example
//! ```rust
//! use tokenectomy::redact::redact_secrets;
//!
//! let raw_log = "Error connecting with DATABASE_URL=postgres://user:secret@localhost:5432/db";
//! let sanitized = redact_secrets(raw_log);
//! assert!(sanitized.contains("[CONNECTION_STRING_REDACTED]"));
//! ```

pub mod cache;
pub mod cli;
pub mod config;
pub mod extractor;
pub mod formatter;
pub mod git;
pub mod mcp;
pub mod provider;
pub mod redact;
pub mod search;

