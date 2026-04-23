//! Pipeline auditors - each module validates a specific aspect of CI/CD configurations
//!
//! # Auditors
//! - `dag`: Dependency graph analysis (cycles, missing deps)
//! - `pinning`: Action/Docker image pinning validation (requires `network` feature)
//! - `secrets`: Secret and environment variable auditing
//! - `syntax`: Pipeline structure validation
//! - `timeout`: Missing job timeout detection
//! - `include`: GitLab CI include: block detection
//! - `schema`: JSON schema-like structural validation

pub mod dag;
pub mod include;
pub mod schema;
pub mod secrets;
pub mod syntax;
pub mod timeout;

#[cfg(feature = "network")]
pub mod pinning;
