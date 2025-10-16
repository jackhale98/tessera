# EDT Usage Examples

This document provides practical examples of using the EDT (Engineering Development Toolkit) backend API.

## Table of Contents

1. [Project Setup](#project-setup)
2. [Task Management](#task-management)
3. [Resource Management](#resource-management)
4. [Calendar Configuration](#calendar-configuration)
5. [Milestone Tracking](#milestone-tracking)
6. [Baseline Creation](#baseline-creation)
7. [Critical Path Analysis](#critical-path-analysis)
8. [Earned Value Management](#earned-value-management)
9. [Complete Project Workflow](#complete-project-workflow)

---

## Project Setup

### Initialize a New Project

```rust
use tessera::{AppState, EdtResult};
use std::path::PathBuf;

fn main() -> EdtResult<()> {
    // Initialize app state with project directory
    let project_path = PathBuf::from("./my_project");
    let app_state = AppState::new(project_path)?;

    println!("Project initialized at: {:?}", app_state.project_root());
    Ok(())
}
```

This creates the following directory structure:
```
my_project/
└── entities/
    ├── tasks/
    ├── milestones/
    ├── resources/
    ├── calendars/
    ├── baselines/
    ├── requirements/
    └── risks/
```

---

## Task Management

### Create a Simple Task

```rust
use chrono::{Utc, Duration};
use tessera::models::TaskType;

fn create_simple_task(app_state: &AppState) -> EdtResult<()> {
    let start = Utc::now();
    let deadline = start + Duration::days(7);

    let task = app_state.entity_manager.create_task(
        "Design System Architecture".to_string(),
        "Create high-level architecture diagrams".to_string(),
        start,
        deadline,
        TaskType::EffortDriven,
    )?;

    println!("Created task: {} (ID: {})", task.name, task.metadata.id);
    Ok(())
}
```

### Create Tasks with Dependencies

```rust
use tessera::models::{TaskDependency, DependencyType};

fn create_dependent_tasks(app_state: &AppState) -> EdtResult<()> {
    let start = Utc::now();

    // Task 1: Design
    let design_task = app_state.entity_manager.create_task(
        "Design Phase".to_string(),
        "System design".to_string(),
        start,
        start + Duration::days(10),
        TaskType::EffortDriven,
    )?;

    // Task 2: Implementation (depends on design)
    let mut impl_task = app_state.entity_manager.create_task(
        "Implementation".to_string(),
        "Code implementation".to_string(),
        start + Duration::days(10),
        start + Duration::days(30),
        TaskType::EffortDriven,
    )?;

    // Add dependency: Implementation starts after Design finishes
    impl_task.dependencies.push(TaskDependency {
        predecessor_id: design_task.metadata.id,
        dependency_type: DependencyType::FinishToStart,
        lag_days: 0.0,
    });

    // Save the updated task
    let impl_task = app_state.entity_manager.update_task(impl_task)?;

    println!("Created dependent tasks");
    Ok(())
}
```

### Update Task Progress

```rust
fn update_task_progress(app_state: &AppState, task_id: &Uuid) -> EdtResult<()> {
    // Get the task
    let mut task = app_state.entity_manager.get_task(task_id)?;

    // Update progress
    task.percent_complete = 0.5;  // 50% complete
    task.actual_cost = Some(5000.0);  // Spent $5,000

    // Record progress history
    task.percent_complete_history.push((Utc::now(), 0.5));

    // Save changes
    let updated = app_state.entity_manager.update_task(task)?;

    println!("Task progress: {}%", updated.percent_complete * 100.0);
    Ok(())
}
```

---

## Resource Management

### Create Human Resources

```rust
use tessera::models::ResourceType;

fn create_team_resources(app_state: &AppState) -> EdtResult<()> {
    // Create senior engineer
    let mut engineer = app_state.entity_manager.create_resource(
        "Alice Johnson".to_string(),
        "Senior Software Engineer".to_string(),
        ResourceType::Labor,
    )?;

    engineer.email = Some("alice@example.com".to_string());
    engineer.bill_rate = Some(150.0);  // $150/hour
    let engineer = app_state.entity_manager.update_resource(engineer)?;

    // Create contractor
    let mut contractor = app_state.entity_manager.create_resource(
        "Bob Smith".to_string(),
        "External Consultant".to_string(),
        ResourceType::FlatCost,
    )?;

    contractor.bill_rate = Some(5000.0);  // $5,000 flat rate
    let contractor = app_state.entity_manager.update_resource(contractor)?;

    println!("Created resources: {} and {}", engineer.name, contractor.name);
    Ok(())
}
```

### Assign Resources to Tasks

```rust
use tessera::models::ResourceAssignment;

fn assign_resources_to_task(
    app_state: &AppState,
    task_id: &Uuid,
    resource_ids: Vec<Uuid>,
) -> EdtResult<()> {
    let mut task = app_state.entity_manager.get_task(task_id)?;

    // Assign engineer for 40 hours
    task.assigned_resources.push(ResourceAssignment {
        resource_id: resource_ids[0],
        allocated_hours: 40.0,
    });

    // Assign contractor for 20 hours
    task.assigned_resources.push(ResourceAssignment {
        resource_id: resource_ids[1],
        allocated_hours: 20.0,
    });

    task.estimated_effort = Some(60.0);  // Total 60 hours

    let updated = app_state.entity_manager.update_task(task)?;

    println!("Assigned {} resources to task", updated.assigned_resources.len());
    Ok(())
}
```

---

## Calendar Configuration

### Create Work Calendars

```rust
use chrono::Weekday;

fn create_calendars(app_state: &AppState) -> EdtResult<()> {
    // Standard 40-hour week
    let standard_calendar = app_state.entity_manager.create_calendar(
        "Standard Work Week".to_string(),
        8.0,  // 8 hours per day
        vec![
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
        ],
    )?;

    // Part-time schedule
    let parttime_calendar = app_state.entity_manager.create_calendar(
        "Part-Time Schedule".to_string(),
        4.0,  // 4 hours per day
        vec![Weekday::Mon, Weekday::Wed, Weekday::Fri],
    )?;

    println!("Created calendars");
    Ok(())
}
```

### Add Holidays to Calendar

```rust
use chrono::NaiveDate;

fn add_holidays_to_calendar(
    app_state: &AppState,
    calendar_id: &Uuid,
) -> EdtResult<()> {
    let mut calendar = app_state.entity_manager.get_calendar(calendar_id)?;

    // Add US holidays
    calendar.holidays = vec![
        NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),   // New Year
        NaiveDate::from_ymd_opt(2025, 7, 4).unwrap(),   // Independence Day
        NaiveDate::from_ymd_opt(2025, 12, 25).unwrap(), // Christmas
    ];

    let updated = app_state.entity_manager.update_calendar(calendar)?;

    println!("Added {} holidays", updated.holidays.len());
    Ok(())
}
```

### Assign Calendar to Resource

```rust
fn assign_calendar_to_resource(
    app_state: &AppState,
    resource_id: &Uuid,
    calendar_id: &Uuid,
) -> EdtResult<()> {
    let mut resource = app_state.entity_manager.get_resource(resource_id)?;

    resource.calendar_id = Some(*calendar_id);

    let updated = app_state.entity_manager.update_resource(resource)?;

    println!("Assigned calendar to resource: {}", updated.name);
    Ok(())
}
```

---

## Milestone Tracking

### Create Project Milestones

```rust
fn create_milestones(app_state: &AppState) -> EdtResult<()> {
    let start = Utc::now();

    // Milestone 1: Design Complete
    let design_milestone = app_state.entity_manager.create_milestone(
        "Design Phase Complete".to_string(),
        "All design documents approved".to_string(),
        start + Duration::days(10),
    )?;

    // Milestone 2: Development Complete
    let dev_milestone = app_state.entity_manager.create_milestone(
        "Development Complete".to_string(),
        "All features implemented and tested".to_string(),
        start + Duration::days(30),
    )?;

    // Milestone 3: Project Delivery
    let delivery_milestone = app_state.entity_manager.create_milestone(
        "Project Delivery".to_string(),
        "Final deliverables handed off".to_string(),
        start + Duration::days(40),
    )?;

    println!("Created 3 project milestones");
    Ok(())
}
```

### Link Milestone to Tasks

```rust
fn link_milestone_to_tasks(
    app_state: &AppState,
    milestone_id: &Uuid,
    task_ids: Vec<Uuid>,
) -> EdtResult<()> {
    let mut milestone = app_state.entity_manager.get_milestone(milestone_id)?;

    // Add all tasks as dependencies
    for task_id in task_ids {
        milestone.dependencies.push(TaskDependency {
            predecessor_id: task_id,
            dependency_type: DependencyType::FinishToStart,
            lag_days: 0.0,
        });
    }

    let updated = app_state.entity_manager.update_milestone(milestone)?;

    println!("Linked {} tasks to milestone", updated.dependencies.len());
    Ok(())
}
```

---

## Baseline Creation

### Create Project Baseline

```rust
fn create_project_baseline(
    app_state: &AppState,
    task_ids: Vec<Uuid>,
) -> EdtResult<()> {
    let baseline = app_state.entity_manager.create_baseline(
        "Q1 2025 Plan".to_string(),
        "Initial approved project plan for Q1".to_string(),
        task_ids.clone(),
    )?;

    println!("Created baseline with {} tasks", baseline.task_ids.len());
    println!("Baseline ID: {}", baseline.metadata.id);
    println!("Created: {}", baseline.created_date);

    Ok(())
}
```

### Compare Current vs Baseline

```rust
fn compare_to_baseline(
    app_state: &AppState,
    baseline_id: &Uuid,
) -> EdtResult<()> {
    let baseline = app_state.entity_manager.get_baseline(baseline_id)?;

    println!("Baseline: {} (created {})", baseline.name, baseline.created_date);
    println!("Tasks in baseline: {}", baseline.task_ids.len());

    // Check each task's current state vs baseline
    for task_id in &baseline.task_ids {
        let task = app_state.entity_manager.get_task(task_id)?;
        println!(
            "  Task: {} - {}% complete",
            task.name,
            task.percent_complete * 100.0
        );
    }

    Ok(())
}
```

---

## Critical Path Analysis

### Run CPM Calculation

```rust
fn calculate_project_critical_path(app_state: &AppState) -> EdtResult<()> {
    let result = app_state.calculation_engine.calculate_critical_path()?;

    println!("Project Duration: {} days", result.project_duration);
    println!("Critical Path Tasks: {}", result.critical_path.len());

    // Show critical tasks
    for task_id in &result.critical_path {
        let task = app_state.entity_manager.get_task(task_id)?;
        println!("  CRITICAL: {}", task.name);
    }

    // Show tasks with slack
    for (task_id, slack) in &result.task_slacks {
        if *slack > 0.0 {
            let task = app_state.entity_manager.get_task(task_id)?;
            println!("  {} - {} days of slack", task.name, slack);
        }
    }

    Ok(())
}
```

### Update Tasks Based on CPM Results

```rust
fn update_critical_path_flags(app_state: &AppState) -> EdtResult<()> {
    let result = app_state.calculation_engine.calculate_critical_path()?;

    // Update all tasks with CPM results
    let task_ids = app_state.entity_manager.list_task_ids()?;

    for task_id in task_ids {
        let mut task = app_state.entity_manager.get_task(&task_id)?;

        // Set critical path flag
        task.is_critical_path = result.critical_path.contains(&task_id);

        // Set slack
        task.slack = result.task_slacks.get(&task_id).copied();

        app_state.entity_manager.update_task(task)?;
    }

    println!("Updated {} tasks with CPM results", task_ids.len());
    Ok(())
}
```

---

## Earned Value Management

### Calculate EVM Metrics

```rust
fn calculate_project_evm(app_state: &AppState) -> EdtResult<()> {
    let metrics = app_state.calculation_engine.calculate_evm()?;

    println!("=== Earned Value Management Metrics ===");
    println!("Planned Value (PV):  ${:.2}", metrics.planned_value);
    println!("Earned Value (EV):   ${:.2}", metrics.earned_value);
    println!("Actual Cost (AC):    ${:.2}", metrics.actual_cost);
    println!();
    println!("Cost Variance (CV):       ${:.2}", metrics.cost_variance);
    println!("Schedule Variance (SV):   ${:.2}", metrics.schedule_variance);
    println!();
    println!("Cost Performance Index (CPI):      {:.2}", metrics.cost_performance_index);
    println!("Schedule Performance Index (SPI):  {:.2}", metrics.schedule_performance_index);
    println!();
    println!("Estimate at Completion (EAC):  ${:.2}", metrics.estimate_at_completion);
    println!("Estimate to Complete (ETC):    ${:.2}", metrics.estimate_to_complete);
    println!("Variance at Completion (VAC):  ${:.2}", metrics.variance_at_completion);

    // Interpretation
    if metrics.cost_performance_index < 1.0 {
        println!("\n⚠️  Project is over budget (CPI < 1.0)");
    } else {
        println!("\n✅  Project is under budget (CPI > 1.0)");
    }

    if metrics.schedule_performance_index < 1.0 {
        println!("⚠️  Project is behind schedule (SPI < 1.0)");
    } else {
        println!("✅  Project is ahead of schedule (SPI > 1.0)");
    }

    Ok(())
}
```

---

## Complete Project Workflow

### Full Project Setup Example

```rust
use chrono::Weekday;
use tessera::{AppState, EdtResult};
use tessera::models::{TaskType, ResourceType, TaskDependency, DependencyType, ResourceAssignment};

fn setup_complete_project() -> EdtResult<()> {
    // 1. Initialize project
    let app_state = AppState::new("./software_project".into())?;

    // 2. Create calendar
    let calendar = app_state.entity_manager.create_calendar(
        "Engineering Calendar".to_string(),
        8.0,
        vec![Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu, Weekday::Fri],
    )?;

    // 3. Create resources
    let mut alice = app_state.entity_manager.create_resource(
        "Alice (Senior Engineer)".to_string(),
        "Team lead".to_string(),
        ResourceType::Labor,
    )?;
    alice.calendar_id = Some(calendar.metadata.id);
    alice.bill_rate = Some(150.0);
    alice = app_state.entity_manager.update_resource(alice)?;

    let mut bob = app_state.entity_manager.create_resource(
        "Bob (Developer)".to_string(),
        "Full-stack developer".to_string(),
        ResourceType::Labor,
    )?;
    bob.calendar_id = Some(calendar.metadata.id);
    bob.bill_rate = Some(100.0);
    bob = app_state.entity_manager.update_resource(bob)?;

    // 4. Create tasks
    let start = Utc::now();

    let mut task1 = app_state.entity_manager.create_task(
        "Requirements Analysis".to_string(),
        "Gather and document requirements".to_string(),
        start,
        start + Duration::days(5),
        TaskType::EffortDriven,
    )?;
    task1.assigned_resources.push(ResourceAssignment {
        resource_id: alice.metadata.id,
        allocated_hours: 40.0,
    });
    task1.estimated_effort = Some(40.0);
    task1.calculated_cost = Some(40.0 * 150.0);  // $6,000
    task1 = app_state.entity_manager.update_task(task1)?;

    let mut task2 = app_state.entity_manager.create_task(
        "Architecture Design".to_string(),
        "Design system architecture".to_string(),
        start + Duration::days(5),
        start + Duration::days(12),
        TaskType::EffortDriven,
    )?;
    task2.dependencies.push(TaskDependency {
        predecessor_id: task1.metadata.id,
        dependency_type: DependencyType::FinishToStart,
        lag_days: 0.0,
    });
    task2.assigned_resources.push(ResourceAssignment {
        resource_id: alice.metadata.id,
        allocated_hours: 40.0,
    });
    task2.estimated_effort = Some(40.0);
    task2.calculated_cost = Some(40.0 * 150.0);  // $6,000
    task2 = app_state.entity_manager.update_task(task2)?;

    let mut task3 = app_state.entity_manager.create_task(
        "Implementation".to_string(),
        "Code the features".to_string(),
        start + Duration::days(12),
        start + Duration::days(32),
        TaskType::EffortDriven,
    )?;
    task3.dependencies.push(TaskDependency {
        predecessor_id: task2.metadata.id,
        dependency_type: DependencyType::FinishToStart,
        lag_days: 0.0,
    });
    task3.assigned_resources.push(ResourceAssignment {
        resource_id: alice.metadata.id,
        allocated_hours: 80.0,
    });
    task3.assigned_resources.push(ResourceAssignment {
        resource_id: bob.metadata.id,
        allocated_hours: 80.0,
    });
    task3.estimated_effort = Some(160.0);
    task3.calculated_cost = Some(80.0 * 150.0 + 80.0 * 100.0);  // $20,000
    task3 = app_state.entity_manager.update_task(task3)?;

    let mut task4 = app_state.entity_manager.create_task(
        "Testing".to_string(),
        "QA and bug fixes".to_string(),
        start + Duration::days(32),
        start + Duration::days(40),
        TaskType::EffortDriven,
    )?;
    task4.dependencies.push(TaskDependency {
        predecessor_id: task3.metadata.id,
        dependency_type: DependencyType::FinishToStart,
        lag_days: 0.0,
    });
    task4.assigned_resources.push(ResourceAssignment {
        resource_id: bob.metadata.id,
        allocated_hours: 40.0,
    });
    task4.estimated_effort = Some(40.0);
    task4.calculated_cost = Some(40.0 * 100.0);  // $4,000
    task4 = app_state.entity_manager.update_task(task4)?;

    // 5. Create milestones
    let milestone1 = app_state.entity_manager.create_milestone(
        "Design Complete".to_string(),
        "All design documents approved".to_string(),
        start + Duration::days(12),
    )?;

    let milestone2 = app_state.entity_manager.create_milestone(
        "Project Delivery".to_string(),
        "Product released to customer".to_string(),
        start + Duration::days(40),
    )?;

    // 6. Create baseline
    let baseline = app_state.entity_manager.create_baseline(
        "Initial Plan".to_string(),
        "Original approved project plan".to_string(),
        vec![
            task1.metadata.id,
            task2.metadata.id,
            task3.metadata.id,
            task4.metadata.id,
        ],
    )?;

    // 7. Run initial calculations
    let cpm_result = app_state.calculation_engine.calculate_critical_path()?;
    println!("Project Duration: {} days", cpm_result.project_duration);
    println!("Critical Tasks: {}", cpm_result.critical_path.len());

    let evm_metrics = app_state.calculation_engine.calculate_evm()?;
    println!("Total Budget: ${:.2}", evm_metrics.planned_value);

    println!("\n✅ Project setup complete!");
    println!("Created:");
    println!("  - 1 calendar");
    println!("  - 2 resources");
    println!("  - 4 tasks");
    println!("  - 2 milestones");
    println!("  - 1 baseline");

    Ok(())
}
```

---

## Frontend Integration (Tauri Commands)

All the above operations are exposed as Tauri commands that can be called from the frontend:

### JavaScript/TypeScript Usage

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// Create a task
const createTask = async () => {
  const response = await invoke('create_task', {
    request: {
      name: "New Feature",
      description: "Implement new feature",
      scheduled_start: new Date().toISOString(),
      deadline: new Date(Date.now() + 7 * 24 * 60 * 60 * 1000).toISOString(),
      task_type: "EffortDriven"
    }
  });
  console.log('Task created:', response);
};

// Calculate critical path
const calculateCPM = async () => {
  const result = await invoke('calculate_critical_path');
  console.log('Project duration:', result.result.project_duration);
  console.log('Critical tasks:', result.result.critical_path);
};

// Calculate EVM
const calculateEVM = async () => {
  const metrics = await invoke('calculate_evm');
  console.log('CPI:', metrics.metrics.cost_performance_index);
  console.log('SPI:', metrics.metrics.schedule_performance_index);
};
```

---

## Available Tauri Commands

### Task Commands
- `create_task`
- `get_task`
- `update_task`
- `delete_task`
- `list_tasks`

### Milestone Commands
- `create_milestone`
- `get_milestone`
- `update_milestone`
- `delete_milestone`
- `list_milestones`

### Resource Commands
- `create_resource`
- `get_resource`
- `update_resource`
- `delete_resource`
- `list_resources`

### Calendar Commands
- `create_calendar`
- `get_calendar`
- `update_calendar`
- `delete_calendar`
- `list_calendars`

### Baseline Commands
- `create_baseline`
- `get_baseline`
- `update_baseline`
- `delete_baseline`
- `list_baselines`

### Calculation Commands
- `calculate_critical_path`
- `calculate_evm`

---

## Best Practices

1. **Always validate dates**: Ensure `scheduled_start < deadline`
2. **Use baselines**: Create baselines before major changes
3. **Track progress regularly**: Update `percent_complete` and `actual_cost`
4. **Run CPM frequently**: After dependency changes
5. **Monitor EVM**: Track CPI and SPI for early warning signs
6. **Assign calendars**: Resources should have calendars for accurate scheduling
7. **Document assumptions**: Use notes fields extensively
8. **Use milestones**: Mark major project phases

---

## Error Handling

All operations return `Result<T, EdtError>`. Always handle errors:

```rust
match app_state.entity_manager.create_task(...) {
    Ok(task) => println!("Created: {}", task.name),
    Err(EdtError::ValidationError(msg)) => eprintln!("Validation failed: {}", msg),
    Err(EdtError::EntityNotFound(id)) => eprintln!("Entity not found: {}", id),
    Err(e) => eprintln!("Error: {}", e),
}
```

---

**For more information, see the API documentation and design documents.**
