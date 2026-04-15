//! Pipeline auditors - each module validates a specific aspect of CI/CD configurations
//!
//! # Auditors
//! - `dag`: Dependency graph analysis (cycles, missing deps)
//! - `pinning`: Action/Docker image pinning validation (requires `network` feature)
//! - `secrets`: Secret and environment variable auditing
//! - `syntax`: Pipeline structure validation
//! - `timeout`: Missing job timeout detection

pub mod dag;
pub mod secrets;
pub mod syntax;
pub mod timeout;

#[cfg(feature = "network")]
pub mod pinning;
