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
cargo run -- quality input:add
cargo run -- quality input:link-req
cargo run -- quality risk:add
cargo run -- quality risk:assess
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
- **Design Inputs/Outputs**: Traceable design artifacts with requirement linking
- **Design Controls**: Review, inspection, test, and validation processes
- **Risk Management**: Categorical risk assessment with probability/impact scoring
- **Monte Carlo Risk Analysis**: Statistical risk assessment with confidence intervals and recommendations
- **Auto-Scoring Engine**: Rule-based risk scoring with configurable thresholds and confidence metrics
- **Traceability Matrix**: Comprehensive link management with gap analysis and coverage reporting

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
```