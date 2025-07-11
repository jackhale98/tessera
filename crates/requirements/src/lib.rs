//! # Tessera Requirements Management
//!
//! This crate provides comprehensive requirements management capabilities for the Tessera
//! engineering toolkit. It handles the complete requirements lifecycle from initial
//! capture through design inputs, outputs, and verification.
//!
//! ## Architecture
//!
//! The requirements module follows a linear traceability model:
//! ```text
//! Requirement → DesignInput → DesignOutput → Verification
//! ```
//!
//! ## Core Concepts
//!
//! - **Requirements**: High-level needs and constraints
//! - **Design Inputs**: Technical specifications derived from requirements
//! - **Design Outputs**: Concrete deliverables that satisfy inputs
//! - **Verifications**: Activities that validate outputs meet requirements
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use tessera_requirements::*;
//! use tessera_core::ProjectContext;
//!
//! # async fn example() -> tessera_core::Result<()> {
//! let project_ctx = ProjectContext::load_from_current_dir()?;
//! let mut repo = RequirementsRepository::load_from_project(&project_ctx)?;
//!
//! // Create a new requirement
//! let requirement = Requirement::new(
//!     "System shall operate at 85°C".to_string(),
//!     "Operating temperature requirement".to_string(),
//!     RequirementCategory::Functional,
//!     Priority::High,
//! );
//!
//! repo.add_requirement(requirement)?;
//! repo.save_to_project(&project_ctx)?;
//! # Ok(())
//! # }
//! ```

// Public API exports
pub mod data;
pub mod repository;
pub mod commands;
pub mod validation;
pub mod traceability;

// Re-export core types for convenience
pub use data::*;
pub use repository::*;
pub use commands::*;
pub use validation::*;
pub use traceability::*;

// Re-export commonly used core types
pub use tessera_core::{Id, Result, ProjectContext, Entity};