//! # Tessera Risk Management
//!
//! This crate provides comprehensive risk management capabilities for the Tessera
//! engineering toolkit. It handles risk identification, assessment, scoring, and
//! mitigation through design controls.
//!
//! ## Architecture
//!
//! The risk management system follows a design-focused approach:
//! ```text
//! Risk → Assessment → DesignControl → Mitigation
//! ```
//!
//! ## Core Concepts
//!
//! - **Risks**: Potential failure modes and their impacts
//! - **Assessment**: Probability and impact scoring with automated calculation
//! - **Design Controls**: Mitigation strategies tied to design outputs
//! - **Scoring**: Configurable risk scoring with normalized calculations
//!
//! ## FMEA Integration
//!
//! The system supports Failure Mode and Effects Analysis (FMEA) methodology:
//! - Failure modes and causes
//! - Effects and impacts
//! - Risk scoring (RPN-style)
//! - Mitigation through design controls
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use tessera_risk::*;
//! use tessera_core::ProjectContext;
//!
//! # async fn example() -> tessera_core::Result<()> {
//! let project_ctx = ProjectContext::load_from_current_dir()?;
//! let mut repo = RiskRepository::load_from_project(&project_ctx)?;
//!
//! // Create a new risk
//! let risk = Risk::new(
//!     "Component Overheating".to_string(),
//!     "Electronic component may overheat under load".to_string(),
//!     RiskCategory::Technical,
//! );
//!
//! repo.add_risk(risk)?;
//! repo.save_to_project(&project_ctx)?;
//! # Ok(())
//! # }
//! ```

// Public API exports
pub mod data;
pub mod repository;
pub mod commands;
pub mod scoring;
pub mod analysis;
pub mod controls;

// Re-export core types for convenience
pub use data::*;
pub use repository::*;
pub use commands::*;
pub use scoring::*;
pub use analysis::*;
pub use controls::*;

// Re-export commonly used core types
pub use tessera_core::{Id, Result, ProjectContext, Entity, Repository};