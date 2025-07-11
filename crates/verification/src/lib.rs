//! # Tessera Verification Framework
//!
//! This crate provides comprehensive verification and testing capabilities for the Tessera
//! engineering toolkit. It handles test procedures, execution, results, and integration
//! with requirements management.
//!
//! ## Architecture
//!
//! The verification system is designed for extensibility and automation:
//! ```text
//! TestProcedure → TestExecution → TestResults → Evidence
//!      ↓
//! Requirements Integration (via tessera-requirements)
//! ```
//!
//! ## Core Concepts
//!
//! - **Test Procedures**: Structured test definitions with steps and expected outcomes
//! - **Test Execution**: Automated or manual execution of procedures
//! - **Test Results**: Captured outcomes with evidence and analysis
//! - **Evidence**: Artifacts supporting test results (logs, screenshots, data)
//! - **Requirements Integration**: Bidirectional linking with requirements verification
//!
//! ## Future Capabilities
//!
//! The framework is designed to support:
//! - Automated test execution
//! - Integration with external test runners
//! - Performance and load testing
//! - Continuous integration workflows
//! - Advanced reporting and analytics
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use tessera_verification::*;
//! use tessera_core::ProjectContext;
//!
//! # async fn example() -> tessera_core::Result<()> {
//! let project_ctx = ProjectContext::load_from_current_dir()?;
//! let mut repo = VerificationRepository::load_from_project(&project_ctx)?;
//!
//! // Create a test procedure
//! let procedure = TestProcedure::new(
//!     "Integration Test".to_string(),
//!     "Test system integration".to_string(),
//!     ProcedureType::Integration,
//! );
//!
//! repo.add_test_procedure(procedure)?;
//! repo.save_to_project(&project_ctx)?;
//! # Ok(())
//! # }
//! ```

// Public API exports
pub mod data;
pub mod repository;
pub mod commands;
pub mod execution;
pub mod reporting;
pub mod integration;

// Re-export core types for convenience
pub use data::*;
pub use repository::*;
pub use commands::*;
pub use execution::*;
pub use reporting::*;
pub use integration::*;

// Re-export commonly used core types
pub use tessera_core::{Id, Result, ProjectContext, Entity, Repository};