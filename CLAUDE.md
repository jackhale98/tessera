# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tessera is a comprehensive CLI-based engineering toolkit implemented as a Rust workspace with modular architecture:

1. **tessera-core** - Shared foundation with ID system, error handling, project management, and Git integration
2. **tessera-quality** - Quality management with requirements, design controls, risk analysis, auto-scoring, and traceability matrix
3. **tessera-pm** - Project management with tasks, resources, scheduling, risk management, issue tracking, baseline management, calendar system, and Gantt chart generation
4. **tessera-tol** - Tolerance analysis with component modeling, stackups, Monte Carlo simulation, sensitivity analysis, and process capability analysis
5. **tessera** - Main CLI application that orchestrates all modules

All data is stored as human-readable RON files in a Git workspace for versioned collaboration.

## Development Commands

### Build and Test Commands
```bash
# Build entire workspace
cargo build

# Run tests for entire workspace
cargo test

# Check code without building
cargo check

# Format all code
cargo fmt

# Lint all code
cargo clippy

# Build and run the main application
cargo run

# Run in release mode
cargo run --release
```

### Application Usage
```bash
# Initialize a new project
cargo run -- init "My Project" --description "Project description"

# Interactive mode (default)
cargo run -- interactive

# Quality management commands
cargo run -- quality req:add
cargo run -- quality req:list
cargo run -- quality req:edit
cargo run -- quality input:add
cargo run -- quality input:list
cargo run -- quality input:edit
cargo run -- quality output:add
cargo run -- quality output:list
cargo run -- quality output:edit
cargo run -- quality verification:add
cargo run -- quality verification:list
cargo run -- quality verification:edit
cargo run -- quality control:add
cargo run -- quality control:list
cargo run -- quality control:edit
cargo run -- quality risk:add
cargo run -- quality risk:list
cargo run -- quality risk:edit
cargo run -- quality risk:assess
cargo run -- quality risk:score
cargo run -- quality trace:matrix
cargo run -- quality dashboard

# Project management commands
cargo run -- pm task:add
cargo run -- pm task:list
cargo run -- pm resource:add
cargo run -- pm milestone:add
cargo run -- pm schedule
cargo run -- pm dashboard

# Tolerance analysis commands
cargo run -- tol component:add
cargo run -- tol feature:add
cargo run -- tol stackup:add
cargo run -- tol analysis:run
cargo run -- tol dashboard

# Project status and validation
cargo run -- status
cargo run -- validate

# Module-specific interactive mode
cargo run -- interactive --module quality
cargo run -- interactive --module pm
cargo run -- interactive --module tol

# Cross-module linking commands
cargo run -- link add
cargo run -- link list
cargo run -- link show
cargo run -- link remove
cargo run -- link validate
```

## Architecture

### Core Foundation (tessera-core)
- **Project Context**: Workspace-aware project management with metadata
- **ID System**: UUID-based entity identification with type-safe wrappers
- **Error Handling**: Comprehensive error types with From implementations for common library errors
- **Git Integration**: Repository operations, commit history, and collaboration features
- **Traits**: Entity, Repository, Linkable, and LinkResolver for extensible module architecture

### Quality Management (tessera-quality)
- **Requirements Management**: Categorized requirements with priorities and acceptance criteria
- **Design Inputs/Outputs**: Traceable design artifacts with automatic requirement linking
- **Verification Management**: Dedicated verification entities that validate design outputs
- **Design Controls**: Review, inspection, test, and validation processes
- **Risk Management**: FMEA-style risk assessment with probability/impact scoring and configurable ranges
- **Risk Scoring**: Simple auto-scoring (probability × impact) with real-time calculation
- **Risk Categorization**: Low/Medium/High/Critical risk levels with configurable thresholds
- **Traceability Matrix**: Comprehensive link management with gap analysis and coverage reporting
- **Interactive Editing**: Full CRUD operations for all quality entities with guided workflows

### Project Management (tessera-pm)
- **Task Management**: Task hierarchy with dependencies, effort tracking, and progress monitoring
- **Resource Management**: Resource allocation, calendars, and utilization tracking
- **Scheduling**: Critical path analysis, Gantt chart generation, and schedule optimization
- **Risk Management**: Project-specific risk registry separate from design risks
- **Issue Tracking**: Issue lifecycle management with SLA definitions and escalation workflows
- **Baseline Management**: Project baseline snapshots with variance analysis and health indicators
- **Calendar System**: Working hours, holidays, exceptions, and resource-specific calendars

### Tolerance Analysis (tessera-tol)
- **Component Modeling**: Feature-based component definitions with tolerances
- **Stackup Analysis**: Dimensional chain analysis with statistical methods
- **Monte Carlo Simulation**: Statistical analysis with multiple distribution types
- **Sensitivity Analysis**: Parameter sensitivity and contribution analysis
- **Process Capability**: Cp, Cpk, and process performance analysis
- **Distribution Engine**: Support for Normal, Uniform, Triangular, and LogNormal distributions

### CLI Application (tessera)
- **Command Structure**: Hierarchical commands with module-specific subcommands
- **Interactive Mode**: inquire-based prompts with fuzzy search and rich formatting
- **Menu Organization**: Hierarchical menu structure with category-based navigation (📋 Manage Entities, 📊 Analysis Tools, ⚙️ Settings, 📈 Dashboard)
- **Entity Management**: Full CRUD operations with guided workflows and automatic linking
- **Async Architecture**: Tokio-based async runtime for future extensibility
- **Rich Output**: Colored terminal output with tables and progress indicators

## Data Model and Persistence

### Project Structure
```
project.ron          # Project metadata and configuration
quality/
  requirements.ron   # Design requirements
  inputs.ron         # Design inputs
  outputs.ron        # Design outputs
  verifications.ron  # Verification activities
  controls.ron       # Design controls
  risks.ron          # Risk registry
pm/
  tasks.ron          # Project tasks
  resources.ron      # Resource definitions
  milestones.ron     # Project milestones
  schedules.ron      # Schedule snapshots
  pm_risks.ron       # Project management risks
  issues.ron         # Issue tracking
  baselines.ron      # Project baselines
  calendars.ron      # Calendar definitions
tol/
  components.ron     # Component definitions
  features.ron       # Feature specifications
  stackups.ron       # Stackup definitions
  analysis.ron       # Analysis results
```

### Key Design Patterns
- **ID-Based Linking**: Cross-module references using UUID-based IDs
- **RON Serialization**: Human-readable, Git-friendly data format
- **Trait-Based Architecture**: Entity and Repository traits for consistent CRUD operations
- **Validation**: Built-in validation for all data structures
- **Error Propagation**: Comprehensive error handling with context preservation

### Quality Module Workflow and Linking
The quality module implements a structured workflow with automatic linking between entities:

#### Entity Relationships
- **Requirements → Design Inputs**: Each design input implements exactly one requirement
- **Design Inputs → Design Outputs**: Each design output satisfies exactly one design input
- **Design Outputs → Verifications**: Each verification validates exactly one design output
- **Design Outputs → Design Controls**: Design controls can be linked to multiple design outputs

#### Interactive Workflow
- **Guided Creation**: When creating design outputs, users must first select the design input being satisfied
- **Automatic Linking**: Links are established automatically during entity creation
- **Validation**: The system ensures required entities exist before allowing dependent entities to be created
- **Edit Capabilities**: Full editing support for requirements and risks, with additional entities coming soon

#### Menu Organization
The quality interactive mode is organized into logical categories:
- **📋 Manage Entities**: CRUD operations for all quality entities
- **📊 Analysis Tools**: Risk assessment, scoring, and traceability matrix
- **⚙️ Settings**: Risk scoring configuration and tolerance thresholds
- **📈 Dashboard**: Quality overview and status

### Module Integration
- **Core Traits**: Shared interfaces for entities, repositories, and linking
- **Cross-Module References**: ID-based links with validation support
- **Git-Aware Operations**: Version control integration for collaboration
- **Extension Points**: Plugin-friendly architecture for future modules

## Key Dependencies

### Core Dependencies
- `clap` - Command-line argument parsing with derive macros
- `inquire` - Interactive prompts with fuzzy search capabilities
- `tokio` - Async runtime for CLI operations
- `serde`/`ron` - Data serialization in human-readable format
- `uuid` - Type-safe ID generation and management
- `git2` - Git repository integration
- `chrono` - Date/time operations with timezone support

### Module-Specific Dependencies
- `rand`/`rand_distr` - Monte Carlo simulation for risk and tolerance analysis
- `statrs` - Statistical analysis functions
- `colored`/`comfy-table` - Rich terminal output formatting
- `petgraph` - Graph algorithms for dependency analysis and critical path calculations

### Development Dependencies
- `tempfile` - Temporary file management for tests
- `assert_cmd` - Command-line testing utilities
- `predicates` - Test assertion predicates

## Extension Guidelines

### Adding New Modules
1. Create new crate in `crates/` directory
2. Implement Entity and Repository traits for data types
3. Add module commands to CLI command structure
4. Implement interactive prompts for user operations
5. Add cross-module linking using ID-based references

### Data Structure Design
- All entities must implement the Entity trait (id, name, validate)
- Use IndexMap for ordered collections that preserve definition order
- Include created/updated timestamps for audit trails
- Support metadata fields for extensibility

### CLI Integration
- Add module-specific subcommands to main CLI
- Implement interactive mode handlers
- Use consistent error handling and user feedback
- Support both command-line and interactive usage patterns

## Quality Module Implementation Status

### Completed Features
- **Requirements Management**: Full CRUD operations with interactive editing
- **Design Inputs/Outputs**: Creation with automatic requirement/input linking
- **Verification Management**: Creation with automatic output linking
- **Risk Management**: Full CRUD operations with scoring and assessment
- **Menu Organization**: Hierarchical menu structure with category-based navigation
- **Guided Workflows**: Step-by-step creation with validation and link establishment
- **Migration System**: Automatic data migration for schema updates

### Current Limitations
- **Editing Support**: Currently available for requirements and risks only
- **Design Input Editing**: Coming soon (placeholder implemented)
- **Design Output Editing**: Coming soon (placeholder implemented)
- **Verification Editing**: Coming soon (placeholder implemented)
- **Design Control Editing**: Coming soon (placeholder implemented)

### Recent Improvements
- **Simplified Linking**: Removed complex manual linking in favor of automatic linking during creation
- **Better UX**: Users select the parent entity first, then create the child entity
- **Validation**: System prevents creating child entities without required parent entities
- **Menu Structure**: Organized into logical categories for better navigation

## Common Issues and Solutions

### Compilation Errors
- **Missing trait implementations**: Ensure all entities implement Entity trait with id(), name(), and validate() methods
- **Type mismatches**: Watch for f32/f64 conflicts - use consistent types throughout calculations
- **Import errors**: Use tessera_core::Id instead of crate::Id in module files
- **DateTime conversions**: Use .date_naive() when converting DateTime<Utc> to NaiveDate

### Running the Application
To test the application after fixes:
```bash
cargo check      # Check for compilation errors
cargo run        # Run in interactive mode
cargo run -- init "Test Project"  # Initialize test project
```

### Key Testing Commands
```bash
# Check specific crate
cargo check -p tessera-quality
cargo check -p tessera-pm  
cargo check -p tessera-tol

# Run tests for specific crate
cargo test -p tessera-core
cargo test -p tessera-quality
cargo test -p tessera-pm
cargo test -p tessera-tol

# Run tests for CLI application
cargo test -p tessera
```