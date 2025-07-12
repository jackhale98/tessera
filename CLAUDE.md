# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tessera is a comprehensive CLI-based engineering toolkit implemented as a Rust workspace with modular architecture:

1. **tessera-core** - Shared foundation with ID system, error handling, project management, and Git integration
2. **tessera-requirements** - Requirements and design artifact management with many-to-many linking support
3. **tessera-risk** - Risk management with design controls, FMEA-style assessment, and industry-standard categorization
4. **tessera-verification** - Verification and testing activities linked to design inputs
5. **tessera-quality** - Legacy quality management module (being phased out in favor of modular approach)
6. **tessera-pm** - Project management with tasks, resources, scheduling, risk management, issue tracking, baseline management, calendar system, and Gantt chart generation
7. **tessera-tol** - Tolerance analysis with component modeling, stackups, Monte Carlo simulation, sensitivity analysis, and process capability analysis
8. **tessera** - Main CLI application that orchestrates all modules

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

# Requirements management commands (new modular approach)
cargo run -- requirements req:add
cargo run -- requirements req:list
cargo run -- requirements req:edit
cargo run -- requirements input:add
cargo run -- requirements input:list
cargo run -- requirements input:edit
cargo run -- requirements output:add
cargo run -- requirements output:list
cargo run -- requirements output:edit
cargo run -- requirements verification:add
cargo run -- requirements verification:list
cargo run -- requirements verification:edit
cargo run -- requirements dashboard
cargo run -- requirements trace:matrix

# Risk management commands (new modular approach)
cargo run -- risk risk:add
cargo run -- risk risk:list
cargo run -- risk risk:edit
cargo run -- risk risk:assess
cargo run -- risk risk:score
cargo run -- risk control:add
cargo run -- risk control:list
cargo run -- risk control:edit
cargo run -- risk dashboard

# Verification management commands (new modular approach)
cargo run -- verification test:add
cargo run -- verification test:list
cargo run -- verification test:run
cargo run -- verification report:generate
cargo run -- verification dashboard

# Legacy quality management commands (deprecated - use modular commands above)
cargo run -- quality dashboard

# Project management commands
cargo run -- pm task:add
cargo run -- pm task:list
cargo run -- pm task:edit
cargo run -- pm task:delete
cargo run -- pm resource:add
cargo run -- pm resource:list
cargo run -- pm resource:edit
cargo run -- pm resource:delete
cargo run -- pm milestone:add
cargo run -- pm milestone:list
cargo run -- pm milestone:edit
cargo run -- pm milestone:delete
cargo run -- pm schedule
cargo run -- pm cost:analysis
cargo run -- pm dashboard

# Legacy project management functionality (data structures implemented, CLI pending)
cargo run -- pm risk:add          # Risk management with comprehensive tracking
cargo run -- pm risk:list         # See tessera-pm/src/risk.rs for implementation
cargo run -- pm risk:edit         # ProjectRisk struct supports mitigation and escalation
cargo run -- pm issue:add         # Issue tracking with SLA management
cargo run -- pm issue:list        # See tessera-pm/src/issue.rs for implementation  
cargo run -- pm issue:edit        # Issue struct supports priority and business impact
cargo run -- pm baseline:create   # Project baseline snapshots with variance analysis
cargo run -- pm baseline:list     # See tessera-pm/src/baseline.rs for implementation
cargo run -- pm baseline:compare  # BaselineComparison struct supports project health
cargo run -- pm calendar:add      # Calendar management with holidays and exceptions
cargo run -- pm calendar:list     # See tessera-pm/src/calendar.rs for implementation
cargo run -- pm calendar:edit     # Calendar struct supports resource-specific calendars

# Tolerance analysis commands
cargo run -- tol component:add
cargo run -- tol component:edit
cargo run -- tol component:list
cargo run -- tol feature:add
cargo run -- tol feature:edit
cargo run -- tol feature:list
cargo run -- tol mate:add
cargo run -- tol mate:edit
cargo run -- tol mate:list
cargo run -- tol stackup:add
cargo run -- tol stackup:edit
cargo run -- tol stackup:list
cargo run -- tol stackup:delete
cargo run -- tol analysis:run
cargo run -- tol analysis:list
cargo run -- tol analysis:delete
cargo run -- tol dashboard

# Project status and validation
cargo run -- status
cargo run -- validate

# Module-specific interactive mode
cargo run -- interactive --module requirements
cargo run -- interactive --module risk
cargo run -- interactive --module verification
cargo run -- interactive --module pm
cargo run -- interactive --module tol

# Enhanced PM Interactive Mode Structure
```
📋 Manage Project
├── ✅ Tasks (Add/List/Edit/Delete)
├── 👥 Resources (Add/List/Edit/Delete)  
└── 🏁 Milestones (Add/List/Edit/Delete + Dependency Management)

📅 Scheduling (Critical Path, Gantt Charts)

⚠️  Project Risk Management (Schedule/Cost Risks)
├── ➕ Add Project Risk
├── 📋 List Project Risks
└── ✏️  Edit Project Risk

🐛 Issue Tracking (SLA Management)
├── ➕ Add Issue
├── 📋 List Issues
└── ✏️  Edit Issue

📊 Baselines (Project Snapshots)
├── 📊 Create Baseline
├── 📋 List Baselines
└── 🔄 Compare Baselines

📅 Calendars (Working Hours/Holidays)
├── ➕ Add Calendar
├── 📋 List Calendars
└── ✏️  Edit Calendar

💰 Cost Analysis (Resource-Based)
📈 Dashboard (Project Overview)
```

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

### Requirements Management (tessera-requirements)
- **Requirements**: Categorized requirements with priorities and stakeholder information
- **Design Inputs**: Traceable design artifacts derived from requirements
- **Design Outputs**: Deliverables that satisfy design inputs (many-to-many relationship)
- **Verifications**: Activities that validate design inputs (many-to-many relationship)
- **Many-to-Many Linking**: Design outputs can link to multiple inputs, verifications can link to multiple inputs
- **Interactive Workflows**: Guided multi-selection for creating and editing linked entities
- **Traceability Matrix**: Full requirements traceability with gap analysis

### Risk Management (tessera-risk)
- **Risk Registry**: Industry-standard risk categorization (Design, Process, Use, Software)
- **Design Controls**: Preventive, Detective, Corrective, Compensating, and Directive controls
- **Risk Assessment**: Probability/impact scoring with configurable matrices
- **Risk Analysis**: Monte Carlo simulation and statistical analysis
- **Auto-Scoring**: Real-time risk score calculation and categorization

### Verification Management (tessera-verification)
- **Test Execution**: Automated and manual test execution frameworks
- **Verification Planning**: Test method selection and acceptance criteria definition
- **Evidence Management**: Documentation and artifact linking
- **Reporting**: Comprehensive verification status and coverage reports
- **Integration**: Cross-module integration with requirements and risk modules

### Project Management (tessera-pm)
- **Task Management**: Task hierarchy with complex dependencies (FinishToStart, StartToStart, FinishToFinish, StartToFinish), effort tracking, progress monitoring, and deletion with dependency cleanup
- **Advanced Scheduling**: Critical path method (CPM) implementation with forward/backward pass, slack calculation, free float analysis, and milestone integration
- **Resource Management**: Resource allocation with percentage-based assignments, hourly rates, skill tracking, utilization analysis, and deletion with assignment cleanup
- **Milestone Support**: Full milestone lifecycle management with dependencies, critical path inclusion, and deletion capabilities
- **Cross-Entity Dependencies**: Tasks and milestones can depend on each other with full relationship type support
- **Scheduling Engine**: Petgraph-based dependency resolution, topological sorting, circular dependency detection, and milestone date integration
- **Cost Analysis**: Comprehensive cost estimation based on resource hourly rates, task progress, and effort allocation with variance tracking
- **Enhanced Displays**: Task listings show dependencies, costs, and progress; critical path shows names and descriptions; schedule includes milestone dates
- **Gantt Chart Generation**: Visual project timelines with critical path highlighting and resource utilization
- **Task Types**: Support for effort-driven, fixed-duration, fixed-work, and milestone tasks with flexible resource scaling
- **Dependency Management**: Enhanced lag/lead time support with multiple relationship types and constraint propagation
- **Progress Tracking**: Effort-weighted completion percentages and real-time progress updates
- **Risk Management**: Project-specific risk registry separate from design risks
- **Issue Tracking**: Issue lifecycle management with SLA definitions and escalation workflows
- **Baseline Management**: Project baseline snapshots with variance analysis and health indicators
- **Calendar System**: Working hours, holidays, exceptions, and resource-specific calendars

### Tolerance Analysis (tessera-tol)
- **Component Modeling**: Feature-based component definitions with engineering metadata
- **Feature Management**: Comprehensive feature specifications with tolerance definitions, feature types (Length, Diameter, Radius, Angle, Position, Surface), categories (External/Internal), and MMC/LMC calculations
- **Mate Relationships**: Assembly mate modeling with fit types (Clearance, Transition, Interference), fit validation, and engineering calculations
- **Stackup Analysis**: Dimensional chain analysis with multiple analysis methods (Worst Case, Root Sum Square, Monte Carlo)
- **Monte Carlo Simulation**: Advanced statistical analysis with multiple distribution types, user-configurable confidence levels, quartile analysis, and CSV data export
- **Sensitivity Analysis**: Feature contribution analysis for design optimization
- **Process Capability**: Cp, Cpk calculations with engineering specification limits
- **Distribution Engine**: Support for Normal, Uniform, Triangular, LogNormal, and Beta distributions
- **Fit Validation**: Automatic mate validation with engineering design rules
- **Data Export**: CSV simulation data export for external analysis and reporting
- **Plot Generation**: Interactive histogram and waterfall plot export in SVG and HTML formats for analysis visualization
- **Visual Analytics**: Quartile-based histogram visualization for Monte Carlo results and feature contribution waterfall charts

### CLI Application (tessera)
- **Modular Commands**: Separate command structures for each module (requirements, risk, verification)
- **Interactive Mode**: inquire-based prompts with multi-selection support and rich formatting
- **Async Architecture**: Tokio-based async runtime for future extensibility
- **Rich Output**: Colored terminal output with tables and progress indicators

## Data Model and Persistence

### Project Structure
```
project.ron          # Project metadata and configuration
requirements/
  requirements.ron   # Design requirements
  design_inputs.ron  # Design inputs
  design_outputs.ron # Design outputs
  verifications.ron  # Verification activities
risk/
  risks.ron          # Risk registry
  controls.ron       # Design controls
verification/
  tests.ron          # Test definitions
  results.ron        # Test execution results
  reports.ron        # Verification reports
quality/             # Legacy quality data (deprecated)
  requirements.ron   
  inputs.ron
  outputs.ron
  verifications.ron
  controls.ron
  risks.ron
pm/
  tasks.ron          # Project tasks with dependency relationships
  resources.ron      # Resource definitions with allocation details
  milestones.ron     # Project milestones with status tracking
  schedules.ron      # Generated schedule snapshots with critical path analysis
  pm_risks.ron       # Project management risks
  issues.ron         # Issue tracking
  baselines.ron      # Project baselines
  calendars.ron      # Calendar definitions
tol/
  components.ron     # Component definitions
  features.ron       # Feature specifications with tolerance data
  mates.ron          # Assembly mate relationships
  stackups.ron       # Stackup definitions
  analyses.ron       # Analysis results with statistics
  plots/             # Exported SVG and HTML plots
data/simulations/    # Monte Carlo CSV export files
```

### Key Design Patterns
- **ID-Based Linking**: Cross-module references using UUID-based IDs
- **Many-to-Many Relationships**: Design outputs link to multiple inputs, verifications link to multiple inputs
- **RON Serialization**: Human-readable, Git-friendly data format
- **Trait-Based Architecture**: Entity and Repository traits for consistent CRUD operations
- **Validation**: Built-in validation for all data structures
- **Error Propagation**: Comprehensive error handling with context preservation

### Entity Relationships (New Modular Approach)
- **Requirements → Design Inputs**: Each design input implements exactly one requirement
- **Design Inputs ↔ Design Outputs**: Many-to-many relationship (outputs can satisfy multiple inputs)
- **Design Inputs ↔ Verifications**: Many-to-many relationship (verifications can validate multiple inputs)
- **Design Controls**: Can be linked to multiple entities across modules

### Interactive Workflow Enhancements
- **Multi-Selection Support**: Users can select multiple entities when creating links
- **Guided Creation**: Step-by-step workflows with validation at each step
- **Field-by-Field Editing**: Comprehensive editing menus for all entity properties
- **Automatic Validation**: Real-time validation during entity creation and modification

### Module Integration
- **Core Traits**: Shared interfaces for entities, repositories, and linking
- **Cross-Module References**: ID-based links with validation support
- **Git-Aware Operations**: Version control integration for collaboration
- **Extension Points**: Plugin-friendly architecture for future modules

## Recent Improvements (Project Management Module)

### Task and Entity Management Enhancements
- **Task/Milestone/Resource Deletion**: Interactive deletion with confirmation prompts and dependency cleanup
- **Enhanced Progress Calculations**: Effort-weighted progress calculations that include individual task progress
- **Improved Task Listing**: Shows dependency details and cross-entity relationships
- **Cross-Entity Dependencies**: Tasks can depend on milestones and vice versa with full dependency type support

### Milestone Dependency Management
- **Comprehensive Dependency Support**: Milestones can depend on both tasks and other milestones
- **Interactive Dependency Assignment**: User-friendly interface for managing milestone dependencies
- **Dependency Type Selection**: Full support for FS, SS, FF, SF dependency types with lag/lead times
- **Dependency Description**: Optional descriptions for dependency relationships

### Schedule and Cost Analysis
- **Enhanced Schedule Generation**: Includes milestone dates and constraints in project schedules
- **Critical Path Enhancement**: Shows task names and descriptions in critical path analysis
- **Comprehensive Cost Analysis**: Resource-based cost estimation with hourly rates and task progress
- **Cross-Entity Integration**: Schedule generation considers both task and milestone dependencies

### Legacy Functionality Integration
- **Project Risk Management**: Full implementation with risk assessment, mitigation actions, and escalation
- **Issue Tracking**: Comprehensive issue management with SLA definitions and escalation workflows
- **Baseline Management**: Project baseline snapshots with variance analysis and health indicators
- **Calendar System**: Working hours, holidays, exceptions, and resource-specific calendar support

### Data Model Improvements
- **Many-to-Many Relationships**: Enhanced support for complex entity relationships
- **Comprehensive Validation**: Real-time validation during entity creation and modification
- **Audit Trails**: Created/updated timestamps for all entities
- **Metadata Support**: Extensible metadata fields for future enhancements

### Interactive Mode Enhancements
- **Field-by-Field Editing**: Comprehensive editing menus for all entity properties
- **Multi-Selection Support**: Ability to select multiple entities when creating links
- **Guided Workflows**: Step-by-step workflows with validation at each step
- **Enhanced Error Handling**: Clear error messages and recovery options

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
- Consider many-to-many relationships when designing entity links

### CLI Integration
- Add module-specific subcommands to main CLI
- Implement interactive mode handlers with multi-selection support
- Use consistent error handling and user feedback
- Support both command-line and interactive usage patterns

## Common Issues and Solutions

### Compilation Errors
- **Missing trait implementations**: Ensure all entities implement Entity trait with id(), name(), and validate() methods
- **Type mismatches**: Watch for f32/f64 conflicts - use consistent types throughout calculations
- **Import errors**: Use tessera_core::Id instead of crate::Id in module files
- **DateTime conversions**: Use .date_naive() when converting DateTime<Utc> to NaiveDate
- **Many-to-many relationship errors**: Ensure Vec<Id> is used for multiple entity links, not single Id

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
cargo check -p tessera-requirements
cargo check -p tessera-risk
cargo check -p tessera-verification
cargo check -p tessera-quality
cargo check -p tessera-pm  
cargo check -p tessera-tol

# Run tests for specific crate
cargo test -p tessera-core
cargo test -p tessera-requirements
cargo test -p tessera-risk
cargo test -p tessera-verification
cargo test -p tessera-quality
cargo test -p tessera-pm
cargo test -p tessera-tol

# Run tests for CLI application
cargo test -p tessera
```

## Migration from Legacy Quality Module

The codebase is transitioning from a monolithic quality module to separate modular crates:

### Legacy vs New Approach
- **Legacy**: Single `tessera-quality` crate with all quality entities
- **New**: Separate `tessera-requirements`, `tessera-risk`, and `tessera-verification` crates

### Data Model Changes
- **One-to-Many → Many-to-Many**: Design outputs and verifications now support multiple linked entities
- **Verification Target Change**: Verifications now link to design inputs instead of design outputs
- **Enhanced Field Support**: Added support for metadata, tags, and extended entity properties

### Command Structure Changes
- **Old**: `cargo run -- quality req:add`
- **New**: `cargo run -- requirements req:add`

### Interactive Mode Enhancements
- **Multi-Selection**: Support for selecting multiple entities during creation/editing
- **Field-by-Field Editing**: Comprehensive editing menus for all entity properties
- **Improved Workflows**: Better user experience with guided entity creation and linking