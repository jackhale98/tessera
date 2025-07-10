# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Blueprint is a comprehensive text-based project management tool built in Rust that enables:
- Project definition via human-readable RON files
- Automated scheduling with dependency resolution and resource allocation
- Gantt chart generation using Mermaid format
- Cost tracking and resource utilization reporting
- Baseline management for project tracking and variance analysis
- Progress tracking with Earned Value Management (EVM) calculations
- Risk and issue management with escalation workflows
- Interactive CLI with comprehensive project management features
- Git-based version control for project collaboration

## Common Commands

### Build and Test
```bash
# Build the project
cargo build

# Run tests
cargo test

# Build and run the application
cargo run

# Run in release mode
cargo run --release
```

### Development Commands
```bash
# Run a specific test
cargo test <test_name>

# Run tests with output
cargo test -- --nocapture

# Check code without building
cargo check

# Format code
cargo fmt

# Run clippy linter
cargo clippy
```

### Application Usage
```bash
# Initialize a new project
cargo run -- init "My Project" --template web-app

# Compute project schedule
cargo run -- schedule project.ron

# Generate Gantt chart
cargo run -- report gantt --format mermaid

# Interactive mode (default when no command specified)
cargo run -- interactive

# Interactive mode with specific project file
cargo run -- interactive project.ron
```

## Architecture

### Core Module Structure
- `core/` - Domain models and business logic
- `scheduling/` - Scheduling engine with dependency resolution and optimization
- `cli/` - Command-line interface and interactive mode
- `reporting/` - Report generation (Gantt charts, cost reports, resource utilization)
- `git/` - Git integration for version control
- `templates/` - Project templates

### Key Components

**Project Definition (core/project.rs)**
- Projects are defined in RON format with comprehensive metadata
- Uses IndexMap for ordered collections preserving definition order
- Supports loading from file via `Project::load_from_file()`
- Includes integrated registries for issues, risks, progress snapshots, and baselines

**Scheduling Engine (scheduling/engine.rs)**
- Computes schedules using dependency graphs (petgraph)
- Handles resource allocation and capacity constraints with calendar integration
- Calculates critical path and project timeline with slack analysis
- Supports multiple task types: EffortDriven, FixedDuration, FixedWork
- Outputs structured Schedule with cost and utilization data

**Interactive CLI (cli/interactive.rs)**
- Comprehensive interactive mode with automatic schedule computation
- Full CRUD operations for all project entities (tasks, resources, milestones, calendars)
- Integrated baseline management with variance analysis
- Progress tracking with EVM calculations (SPI/CPI metrics)
- Risk and issue management with priority-based workflows
- Color-coded displays for project health indicators

**Baseline Management (core/baseline_manager.rs)**
- Creates project baselines at specific points in time
- Supports multiple baseline types (Initial, Approved, Current)
- Provides variance analysis between baselines and current state
- Git-friendly RON storage for baseline snapshots

**Progress Tracking (core/progress.rs)**
- Progress snapshots capture actual vs. planned performance
- Earned Value Management with full PV/EV/AC calculations
- SPI and CPI metrics for schedule and cost performance
- Trend analysis across multiple progress snapshots

**Risk and Issue Management (core/risk.rs, core/issue.rs)**
- Comprehensive risk registry with probability/impact matrices
- Issue tracking with priority, severity, and escalation workflows
- Integration with tasks and milestones for impact assessment
- Metrics and reporting for project health monitoring

### Data Flow and Integration

1. **Project Initialization**: RON files parsed into Project structs with integrated registries
2. **Schedule Computation**: Automatic scheduling on startup and after any changes
3. **Baseline Creation**: Project state captured for future variance analysis
4. **Progress Tracking**: Regular snapshots enable EVM calculations and trend analysis
5. **Risk/Issue Management**: Continuous monitoring with escalation rules
6. **Reporting**: Multiple output formats (Mermaid, Markdown, tables) for stakeholder communication

### Calendar and Resource Management

- Default calendar with configurable working hours and holidays
- Resource-specific calendars for accurate capacity planning
- Calendar exceptions for project-specific scheduling constraints
- Integration with task scheduling for realistic timeline calculations

### Task and Dependency Management

- Support for all four dependency types: FS, SS, FF, SF
- Lag/lead time support for complex scheduling scenarios
- Multiple resource assignments per task with allocation percentages
- Critical path analysis with slack calculations

## Development Notes

### Core Dependencies
- `anyhow` for error handling throughout the application
- `chrono` for date/time operations and calendar management
- `petgraph` for dependency graph analysis and critical path calculations
- `ron` for RON serialization/deserialization
- `indexmap` for ordered collections that preserve definition order
- `colored` for terminal output formatting
- `inquire` for interactive CLI prompts and validation
- `clap` for command-line argument parsing

### Interactive Mode Features
The interactive mode provides a comprehensive project management interface:
- Automatic schedule computation after any project changes
- Real-time validation and error handling
- Color-coded status indicators for project health
- Progressive disclosure of advanced features
- Contextual help and guided workflows

### Extension Points
The architecture supports extensibility through trait-based interfaces:
- `SchedulingAlgorithm` trait for pluggable optimization algorithms
- Baseline comparison strategies for different analysis approaches
- Report generators for custom output formats
- Risk/issue escalation rules for organization-specific workflows

The codebase follows clean architecture principles with clear separation between domain logic, scheduling algorithms, and presentation layers, making it straightforward to extend with new features or integrate with external systems.