use tessera_core::{ProjectContext, Result, Id};
use colored::Colorize;
use comfy_table::{Table, Cell, Attribute, Color};
use inquire::{Text, Select, Confirm, MultiSelect};
use tessera_team::{TeamRepository, TeamMember, Role, Team, TeamType, GitApprovalAuthority};
use std::path::Path;
use std::fs;

use crate::TeamCommands;
use crate::impact_service::get_impact_service;
use tessera_impact::{ModuleType, ChangeType};

pub async fn execute_team_command(command: TeamCommands, project_ctx: ProjectContext) -> Result<()> {
    match command {
        TeamCommands::AddMember => {
            add_team_member(&project_ctx).await?;
        },
        
        TeamCommands::ListMembers => {
            list_team_members(&project_ctx).await?;
        },
        
        TeamCommands::EditMember => {
            println!("{}", "Edit team member not yet implemented".yellow());
        },
        
        TeamCommands::DeactivateMember => {
            println!("{}", "Deactivate team member not yet implemented".yellow());
        },
        
        TeamCommands::AddRole => {
            add_role(&project_ctx).await?;
        },
        
        TeamCommands::ListRoles => {
            list_roles(&project_ctx).await?;
        },
        
        TeamCommands::EditRole => {
            println!("{}", "Edit role not yet implemented".yellow());
        },
        
        TeamCommands::AssignRole => {
            println!("{}", "Assign role not yet implemented".yellow());
        },
        
        TeamCommands::RemoveRole => {
            println!("{}", "Remove role not yet implemented".yellow());
        },
        
        TeamCommands::AddTeam => {
            add_team(&project_ctx).await?;
        },
        
        TeamCommands::ListTeams => {
            list_teams(&project_ctx).await?;
        },
        
        TeamCommands::EditTeam => {
            println!("{}", "Edit team not yet implemented".yellow());
        },
        
        TeamCommands::AddMemberToTeam => {
            println!("{}", "Add member to team not yet implemented".yellow());
        },
        
        TeamCommands::RemoveMemberFromTeam => {
            println!("{}", "Remove member from team not yet implemented".yellow());
        },
        
        TeamCommands::SetTeamLead => {
            println!("{}", "Set team lead not yet implemented".yellow());
        },
        
        TeamCommands::Validate => {
            println!("{}", "Team Validation".bold().blue());
            validate_team_configuration(&project_ctx).await?;
        },
        
        TeamCommands::GitSync => {
            println!("{}", "Git Team Sync".bold().blue());
            println!("Git team synchronization not yet implemented.");
            println!("This will sync team data with Git teams and usernames.");
        },
        
        TeamCommands::GenerateCodeowners => {
            println!("{}", "Generate CODEOWNERS".bold().blue());
            println!("CODEOWNERS generation not yet implemented.");
            println!("This will generate CODEOWNERS files based on approval rules.");
        },
        
        TeamCommands::Dashboard => {
            println!("{}", "Team Dashboard".bold().blue());
            show_team_dashboard(&project_ctx).await?;
        },
    }
    
    Ok(())
}

async fn validate_team_configuration(project_ctx: &ProjectContext) -> Result<()> {
    println!("Validating team configuration...");
    
    let team_dir = project_ctx.root_path.join("team");
    
    if !team_dir.exists() {
        println!("  {} Team directory does not exist: {}", "⚠".yellow(), team_dir.display());
        println!("  {} Creating team directory...", "ℹ".blue());
        std::fs::create_dir_all(&team_dir)?;
        println!("  {} Team directory created", "✓".green());
    } else {
        println!("  {} Team directory exists", "✓".green());
    }
    
    // Check for team data files
    let files = vec![
        ("members.ron", "Team members data"),
        ("roles.ron", "Role definitions"),
        ("teams.ron", "Team structure"),
        ("approval_rules.ron", "Approval rules"),
    ];
    
    for (filename, description) in files {
        let file_path = team_dir.join(filename);
        if file_path.exists() {
            println!("  {} {} exists", "✓".green(), description);
        } else {
            println!("  {} {} missing: {}", "⚠".yellow(), description, filename);
        }
    }
    
    println!("{}", "Team validation complete!".green());
    Ok(())
}

async fn show_team_dashboard(_project_ctx: &ProjectContext) -> Result<()> {
    println!("Team management dashboard coming soon!");
    println!();
    
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("Module").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Status").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Description").add_attribute(Attribute::Bold).fg(Color::Blue),
    ]);
    
    let modules = vec![
        ("Team Members", "Planned", "Individual team member profiles and contact information"),
        ("Roles & Permissions", "Planned", "Role-based access control and approval authorities"),
        ("Teams & Groups", "Planned", "Organizational team structure and hierarchy"),
        ("Git Integration", "Planned", "Git team sync and CODEOWNERS generation"),
        ("Approval Workflows", "Planned", "Automated approval chain management"),
    ];
    
    for (module, status, description) in modules {
        let status_cell = match status {
            "Planned" => Cell::new(status).fg(Color::Yellow),
            "Active" => Cell::new(status).fg(Color::Green),
            _ => Cell::new(status),
        };
        
        table.add_row(vec![
            Cell::new(module),
            status_cell,
            Cell::new(description),
        ]);
    }
    
    println!("{}", table);
    
    println!();
    println!("{}", "📋 Available Commands:".bold());
    println!("  tessera team member:add    - Add new team member");
    println!("  tessera team role:add      - Create new role");
    println!("  tessera team team:add      - Create new team");
    println!("  tessera team validate      - Validate team configuration");
    println!("  tessera team dashboard     - Show this dashboard");
    
    Ok(())
}

async fn add_team_member(project_ctx: &ProjectContext) -> Result<()> {
    println!("{}", "Add Team Member".bold().blue());
    
    let first_name = Text::new("First name:")
        .with_help_message("Enter the team member's first name")
        .prompt()?;
    
    let last_name = Text::new("Last name:")
        .with_help_message("Enter the team member's last name")
        .prompt()?;
    
    let email = Text::new("Email:")
        .with_help_message("Enter the team member's email address")
        .prompt()?;
    
    let department = Text::new("Department:")
        .with_help_message("Enter the department")
        .with_default("Engineering")
        .prompt()?;
    
    let job_title = Text::new("Job title:")
        .with_help_message("Enter the job title")
        .with_default("Engineer")
        .prompt()?;
    
    let git_username = Text::new("Git username:")
        .with_help_message("Enter Git username (optional)")
        .with_default("")
        .prompt()?;
    
    // For now, we'll create a temporary role ID. In a full implementation,
    // we would let the user select from existing roles
    let temp_role_id = Id::new();
    
    let mut member = TeamMember::new(first_name, last_name, email, job_title, department, temp_role_id);
    
    if !git_username.is_empty() {
        member.git_username = Some(git_username);
    }
    
    let mut repository = TeamRepository::load_from_project(project_ctx)?;
    repository.add_team_member(member.clone())?;
    repository.save_to_project(project_ctx)?;
    
    // Trigger automatic impact analysis
    if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
        let _ = service.on_entity_changed(
            &member,
            ModuleType::Team,
            "TeamMember".to_string(),
            ChangeType::Created,
            &project_ctx,
        ).await;
    }
    
    println!("{} Team member {} {} added successfully!", 
             "✓".green(), member.first_name, member.last_name);
    
    Ok(())
}

async fn list_team_members(project_ctx: &ProjectContext) -> Result<()> {
    println!("{}", "Team Members".bold().blue());
    
    let repository = TeamRepository::load_from_project(project_ctx)?;
    let members = repository.get_team_members();
    
    if members.is_empty() {
        println!("No team members found.");
        println!("Use 'tessera team member:add' to add team members.");
        return Ok(());
    }
    
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("Name").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Email").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Department").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Job Title").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Git Username").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Status").add_attribute(Attribute::Bold).fg(Color::Blue),
    ]);
    
    for member in members.values() {
        let status_cell = if member.active {
            Cell::new("Active").fg(Color::Green)
        } else {
            Cell::new("Inactive").fg(Color::Red)
        };
        
        table.add_row(vec![
            Cell::new(format!("{} {}", member.first_name, member.last_name)),
            Cell::new(&member.email),
            Cell::new(&member.department),
            Cell::new(&member.job_title),
            Cell::new(member.git_username.as_deref().unwrap_or("-")),
            status_cell,
        ]);
    }
    
    println!("{}", table);
    println!("Total: {} team members", members.len());
    
    Ok(())
}

async fn add_role(project_ctx: &ProjectContext) -> Result<()> {
    println!("{}", "Add Role".bold().blue());
    
    let name = Text::new("Role name:")
        .with_help_message("Enter the role name (e.g., 'Lead Engineer', 'Quality Manager')")
        .prompt()?;
    
    let description = Text::new("Description:")
        .with_help_message("Enter a description of this role")
        .with_default("Role description")
        .prompt()?;
    
    let approval_authority = Confirm::new("Has Git approval authority?")
        .with_help_message("Can this role approve Git changes?")
        .with_default(false)
        .prompt()?;
    
    let git_approval_authority = if approval_authority {
        let paths = select_approval_paths_interactive(project_ctx).await?;
        
        GitApprovalAuthority {
            can_approve_paths: paths,
            approval_contexts: Vec::new(),
            max_cost_approval: None,
            max_schedule_impact_days: None,
        }
    } else {
        GitApprovalAuthority {
            can_approve_paths: Vec::new(),
            approval_contexts: Vec::new(),
            max_cost_approval: None,
            max_schedule_impact_days: None,
        }
    };
    
    let mut role = Role::new(name, description);
    role.git_approval_authority = git_approval_authority;
    
    let mut repository = TeamRepository::load_from_project(project_ctx)?;
    repository.add_role(role.clone())?;
    repository.save_to_project(project_ctx)?;
    
    // Trigger automatic impact analysis
    if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
        let _ = service.on_entity_changed(
            &role,
            ModuleType::Team,
            "Role".to_string(),
            ChangeType::Created,
            &project_ctx,
        ).await;
    }
    
    println!("{} Role '{}' added successfully!", "✓".green(), role.name);
    
    Ok(())
}

async fn list_roles(project_ctx: &ProjectContext) -> Result<()> {
    println!("{}", "Roles".bold().blue());
    
    let repository = TeamRepository::load_from_project(project_ctx)?;
    let roles = repository.get_roles();
    
    if roles.is_empty() {
        println!("No roles found.");
        println!("Use 'tessera team role:add' to create roles.");
        return Ok(());
    }
    
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("Name").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Git Approval").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Approval Paths").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Description").add_attribute(Attribute::Bold).fg(Color::Blue),
    ]);
    
    for role in roles.values() {
        let approval_cell = if !role.git_approval_authority.can_approve_paths.is_empty() {
            Cell::new("Yes").fg(Color::Green)
        } else {
            Cell::new("No").fg(Color::Red)
        };
        
        let paths = if role.git_approval_authority.can_approve_paths.is_empty() {
            "-".to_string()
        } else {
            role.git_approval_authority.can_approve_paths.join(", ")
        };
        
        table.add_row(vec![
            Cell::new(&role.name),
            approval_cell,
            Cell::new(paths),
            Cell::new(&role.description),
        ]);
    }
    
    println!("{}", table);
    println!("Total: {} roles", roles.len());
    
    Ok(())
}

async fn add_team(project_ctx: &ProjectContext) -> Result<()> {
    println!("{}", "Add Team".bold().blue());
    
    let name = Text::new("Team name:")
        .with_help_message("Enter the team name (e.g., 'Engineering', 'Quality Assurance')")
        .prompt()?;
    
    let description = Text::new("Description:")
        .with_help_message("Enter a description of this team")
        .with_default("Team description")
        .prompt()?;
    
    let team_types = vec!["Engineering", "Quality", "Manufacturing", "ProjectManagement", "Safety", "Test", "Management"];
    let team_type_str = Select::new("Team type:", team_types)
        .with_help_message("Select the type of team")
        .prompt()?;
    
    let team_type = match team_type_str {
        "Engineering" => TeamType::Engineering,
        "Quality" => TeamType::Quality,
        "Manufacturing" => TeamType::Manufacturing,
        "ProjectManagement" => TeamType::ProjectManagement,
        "Safety" => TeamType::Safety,
        "Test" => TeamType::Test,
        "Management" => TeamType::Management,
        _ => TeamType::Engineering,
    };
    
    let mut team = Team::new(name, description, team_type);
    
    // Load repository to check for existing members
    let repository = TeamRepository::load_from_project(project_ctx)?;
    let members = repository.get_team_members();
    
    if !members.is_empty() {
        let add_members = Confirm::new("Add team members now?")
            .with_help_message("You can add members to this team now or later")
            .with_default(false)
            .prompt()?;
        
        if add_members {
            let member_options: Vec<String> = members.values()
                .map(|m| format!("{} {} ({})", m.first_name, m.last_name, m.email))
                .collect();
            
            if !member_options.is_empty() {
                let selected_members = MultiSelect::new("Select team members:", member_options)
                    .with_help_message("Select members to add to this team")
                    .prompt()?;
                
                for selected in selected_members {
                    // Find the member ID based on the selected string
                    for (id, member) in members.iter() {
                        let member_str = format!("{} {} ({})", member.first_name, member.last_name, member.email);
                        if member_str == selected {
                            team.members.push(*id);
                            break;
                        }
                    }
                }
            }
        }
    }
    
    let mut repository = TeamRepository::load_from_project(project_ctx)?;
    repository.add_team(team.clone())?;
    repository.save_to_project(project_ctx)?;
    
    // Trigger automatic impact analysis
    if let Ok(mut service) = std::panic::catch_unwind(|| get_impact_service()) {
        let _ = service.on_entity_changed(
            &team,
            ModuleType::Team,
            "Team".to_string(),
            ChangeType::Created,
            &project_ctx,
        ).await;
    }
    
    println!("{} Team '{}' added successfully!", "✓".green(), team.name);
    if !team.members.is_empty() {
        println!("  {} team members added", team.members.len());
    }
    
    Ok(())
}

async fn list_teams(project_ctx: &ProjectContext) -> Result<()> {
    println!("{}", "Teams".bold().blue());
    
    let repository = TeamRepository::load_from_project(project_ctx)?;
    let teams = repository.get_teams();
    
    if teams.is_empty() {
        println!("No teams found.");
        println!("Use 'tessera team team:add' to create teams.");
        return Ok(());
    }
    
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("Name").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Type").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Members").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Lead").add_attribute(Attribute::Bold).fg(Color::Blue),
        Cell::new("Description").add_attribute(Attribute::Bold).fg(Color::Blue),
    ]);
    
    for team in teams.values() {
        let lead_info = if let Some(_lead_id) = team.lead_id {
            "Yes".to_string()
        } else {
            "No".to_string()
        };
        
        table.add_row(vec![
            Cell::new(&team.name),
            Cell::new(format!("{:?}", team.team_type)),
            Cell::new(team.members.len().to_string()),
            Cell::new(lead_info),
            Cell::new(&team.description),
        ]);
    }
    
    println!("{}", table);
    println!("Total: {} teams", teams.len());
    
    Ok(())
}

// Directory tree structure for approval path selection
#[derive(Debug, Clone)]
struct DirectoryNode {
    name: String,
    path: String,
    is_directory: bool,
    children: Vec<DirectoryNode>,
}

impl DirectoryNode {
    fn new(name: String, path: String, is_directory: bool) -> Self {
        Self {
            name,
            path,
            is_directory,
            children: Vec::new(),
        }
    }
}

async fn select_approval_paths_interactive(project_ctx: &ProjectContext) -> Result<Vec<String>> {
    println!("{}", "Select Approval Paths".bold().blue());
    println!("Choose directories and files this role can approve in Git workflows.\n");
    
    // First, offer some common presets
    let preset_options = vec![
        "📁 Browse repository structure",
        "⭐ All files (*)",
        "🔧 Source code only (src/*, crates/*)",
        "📚 Documentation only (docs/*, *.md)",
        "🧪 Tests only (tests/*, *_test.rs)",
        "⚙️  Configuration files (*.toml, *.yml, *.json)",
        "🎯 Custom paths (manual entry)",
    ];
    
    let choice = Select::new("How would you like to select approval paths?", preset_options)
        .with_help_message("Choose a method to define which files this role can approve")
        .prompt()?;
    
    match choice {
        "📁 Browse repository structure" => {
            browse_directory_tree(project_ctx).await
        },
        "⭐ All files (*)" => {
            Ok(vec!["*".to_string()])
        },
        "🔧 Source code only (src/*, crates/*)" => {
            Ok(vec!["src/*".to_string(), "crates/*".to_string(), "*.rs".to_string()])
        },
        "📚 Documentation only (docs/*, *.md)" => {
            Ok(vec!["docs/*".to_string(), "*.md".to_string(), "README*".to_string()])
        },
        "🧪 Tests only (tests/*, *_test.rs)" => {
            Ok(vec!["tests/*".to_string(), "*_test.rs".to_string(), "test_*".to_string()])
        },
        "⚙️  Configuration files (*.toml, *.yml, *.json)" => {
            Ok(vec!["*.toml".to_string(), "*.yml".to_string(), "*.yaml".to_string(), "*.json".to_string()])
        },
        "🎯 Custom paths (manual entry)" => {
            let paths = Text::new("Enter approval paths (comma-separated):")
                .with_help_message("Enter file paths this role can approve (e.g., 'src/*,docs/*')")
                .with_default("*")
                .prompt()?;
            Ok(paths.split(',').map(|s| s.trim().to_string()).collect())
        },
        _ => Ok(vec!["*".to_string()]),
    }
}

async fn browse_directory_tree(project_ctx: &ProjectContext) -> Result<Vec<String>> {
    println!("{}", "Repository Directory Browser".bold().green());
    println!("Scanning repository structure...\n");
    
    // Build directory tree
    let tree = build_directory_tree(&project_ctx.root_path)?;
    
    // Get all paths from the tree for selection
    let mut all_paths = Vec::new();
    collect_all_paths(&tree, &mut all_paths);
    
    if all_paths.is_empty() {
        println!("{} No directories found in repository.", "⚠".yellow());
        return Ok(vec!["*".to_string()]);
    }
    
    // Display the tree structure
    display_directory_tree(&tree, 0);
    
    println!("\n{}", "Select paths for approval permissions:".bold());
    
    // Let user select multiple paths
    let selected_paths = MultiSelect::new("Select directories and files:", all_paths)
        .with_help_message("Use space to select/deselect, Enter to confirm selection")
        .prompt()?;
    
    if selected_paths.is_empty() {
        println!("{} No paths selected, defaulting to all files (*).", "ℹ".blue());
        Ok(vec!["*".to_string()])
    } else {
        // Convert selected paths to glob patterns
        let mut patterns = Vec::new();
        for path in selected_paths {
            if path.ends_with('/') {
                // Directory - add wildcard
                patterns.push(format!("{}*", path));
            } else {
                // File - add as-is
                patterns.push(path);
            }
        }
        
        println!("\n{} Selected approval patterns:", "✓".green());
        for pattern in &patterns {
            println!("  • {}", pattern);
        }
        
        Ok(patterns)
    }
}

fn build_directory_tree(root_path: &Path) -> Result<DirectoryNode> {
    let mut root = DirectoryNode::new(
        root_path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        String::new(),
        true,
    );
    
    build_tree_recursive(root_path, &mut root, 0)?;
    Ok(root)
}

fn build_tree_recursive(current_path: &Path, node: &mut DirectoryNode, depth: usize) -> Result<()> {
    // Limit depth to avoid overwhelming output
    if depth > 3 {
        return Ok(());
    }
    
    // Skip hidden directories and common ignore patterns
    let skip_dirs = [".git", ".vscode", "target", "node_modules", ".idea", "dist", "build"];
    
    if let Ok(entries) = fs::read_dir(current_path) {
        let mut entries: Vec<_> = entries.collect::<std::result::Result<Vec<_>, _>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        
        for entry in entries {
            let path = entry.path();
            let name = path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            
            // Skip hidden files and directories
            if name.starts_with('.') && !name.ends_with(".md") && !name.ends_with(".toml") {
                continue;
            }
            
            // Skip common ignore patterns
            if skip_dirs.contains(&name.as_str()) {
                continue;
            }
            
            let relative_path = path.strip_prefix(&current_path.parent().unwrap_or(current_path))
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            
            let is_directory = path.is_dir();
            let mut child_node = DirectoryNode::new(name, relative_path, is_directory);
            
            if is_directory {
                build_tree_recursive(&path, &mut child_node, depth + 1)?;
            }
            
            node.children.push(child_node);
        }
    }
    
    Ok(())
}

fn display_directory_tree(node: &DirectoryNode, depth: usize) {
    let indent = "  ".repeat(depth);
    let icon = if node.is_directory {
        if node.children.is_empty() {
            "📁"
        } else {
            "📂"
        }
    } else {
        "📄"
    };
    
    if depth == 0 {
        println!("{}🏠 {} (project root)", indent, node.name.blue().bold());
    } else {
        let display_name = if node.is_directory {
            format!("{}/", node.name).blue()
        } else {
            node.name.normal()
        };
        println!("{}{} {}", indent, icon, display_name);
    }
    
    // Limit display depth to keep output manageable
    if depth < 3 {
        for child in &node.children {
            display_directory_tree(child, depth + 1);
        }
    } else if !node.children.is_empty() {
        println!("{}  📋 ... ({} more items)", "  ".repeat(depth + 1), node.children.len());
    }
}

fn collect_all_paths(node: &DirectoryNode, paths: &mut Vec<String>) {
    if !node.path.is_empty() {
        if node.is_directory {
            paths.push(format!("{}/", node.path));
        } else {
            paths.push(node.path.clone());
        }
    }
    
    for child in &node.children {
        collect_all_paths(child, paths);
    }
}