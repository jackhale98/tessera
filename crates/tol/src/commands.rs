use crate::data::*;
use crate::repository::ToleranceRepository;
use crate::analysis::ToleranceAnalyzer;
use tessera_core::{ProjectContext, Id, Result};
use inquire::{Select, Text, Confirm};

pub struct ToleranceCommands {
    repository: ToleranceRepository,
    project_context: ProjectContext,
}

impl ToleranceCommands {
    pub fn new(project_context: ProjectContext) -> Result<Self> {
        let tol_dir = project_context.module_path("tol");
        let repository = ToleranceRepository::load_from_directory(&tol_dir)?;
        
        Ok(Self {
            repository,
            project_context,
        })
    }
    
    pub async fn add_component_interactive(&mut self) -> Result<()> {
        let name = Text::new("Component name:")
            .with_help_message("Enter a name for the component")
            .prompt()?;
        
        let description = Text::new("Description:")
            .with_help_message("Describe the component")
            .prompt()?;
        
        let part_number = Text::new("Part number (optional):")
            .with_help_message("Leave blank if not applicable")
            .prompt()?;
        
        let mut component = Component::new(name, description);
        if !part_number.is_empty() {
            component.part_number = Some(part_number);
        }
        
        self.repository.add_component(component.clone())?;
        
        let tol_dir = self.project_context.module_path("tol");
        self.repository.save_to_directory(&tol_dir)?;
        
        println!("✓ Component '{}' added successfully!", component.name);
        println!("ID: {}", component.id);
        
        Ok(())
    }
    
    pub async fn add_feature_interactive(&mut self) -> Result<()> {
        let components = self.repository.get_components();
        if components.is_empty() {
            println!("No components found. Add components first.");
            return Ok(());
        }
        
        let component_options: Vec<String> = components.iter()
            .map(|c| format!("{} - {}", c.name, c.description))
            .collect();
        
        let component_selection = Select::new("Select component:", component_options.clone()).prompt()?;
        let component_index = component_options.iter().position(|x| x == &component_selection).unwrap();
        let selected_component = &components[component_index];
        
        let name = Text::new("Feature name:")
            .prompt()?;
        
        let description = Text::new("Description:")
            .prompt()?;
        
        let feature_types = vec![
            "Length",
            "Diameter", 
            "Radius",
            "Angle",
            "Position",
            "Surface",
            "Other",
        ];
        
        let feature_type_str = Select::new("Feature type:", feature_types).prompt()?;
        let feature_type = match feature_type_str {
            "Length" => FeatureType::Length,
            "Diameter" => FeatureType::Diameter,
            "Radius" => FeatureType::Radius,
            "Angle" => FeatureType::Angle,
            "Position" => FeatureType::Position,
            "Surface" => FeatureType::Surface,
            _ => FeatureType::Other
        };

        let feature_categories = vec![
            "External",
            "Internal",
        ];
        
        let feature_category_str = Select::new("Feature category:", feature_categories)
            .with_help_message("External (shafts, pins) or Internal (holes, slots)")
            .prompt()?;
        let feature_category = match feature_category_str {
            "External" => FeatureCategory::External,
            "Internal" => FeatureCategory::Internal,
            _ => FeatureCategory::External
        };
        
        let nominal_str = Text::new("Nominal value:")
            .with_default("0.0")
            .prompt()?;
        let nominal: f64 = nominal_str.parse().unwrap_or(0.0);
        
        let plus_tol_str = Text::new("Plus tolerance:")
            .with_default("0.1")
            .prompt()?;
        let plus_tolerance: f64 = plus_tol_str.parse().unwrap_or(0.1);
        
        let minus_tol_str = Text::new("Minus tolerance:")
            .with_default("0.1")
            .prompt()?;
        let minus_tolerance: f64 = minus_tol_str.parse().unwrap_or(0.1);
        
        let drawing_location = Text::new("Drawing location (optional):")
            .with_help_message("Reference to drawing location, e.g., 'View A-A, Detail B'")
            .prompt()?;
        
        let mut feature = Feature::new(name, description, selected_component.id, feature_type, feature_category, nominal);
        feature.tolerance.plus = plus_tolerance;
        feature.tolerance.minus = minus_tolerance;
        
        if !drawing_location.is_empty() {
            feature.drawing_location = Some(drawing_location);
        }
        
        // Ask about distribution type
        let distribution_types = vec![
            "Normal",
            "Uniform",
            "Triangular",
            "LogNormal",
            "Beta",
        ];
        
        let distribution_str = Select::new("Distribution type:", distribution_types).prompt()?;
        
        let distribution_type = match distribution_str {
            "Normal" => ToleranceDistribution::Normal,
            "Uniform" => ToleranceDistribution::Uniform,
            "Triangular" => ToleranceDistribution::Triangular,
            "LogNormal" => ToleranceDistribution::LogNormal,
            "Beta" => {
                let alpha_str = Text::new("Beta alpha parameter:")
                    .with_default("2.0")
                    .prompt()?;
                let alpha: f64 = alpha_str.parse().unwrap_or(2.0);
                
                let beta_str = Text::new("Beta beta parameter:")
                    .with_default("2.0")
                    .prompt()?;
                let beta: f64 = beta_str.parse().unwrap_or(2.0);
                
                ToleranceDistribution::Beta { alpha, beta }
            }
            _ => ToleranceDistribution::Normal,
        };
        
        feature.update_distribution(distribution_type);
        
        self.repository.add_feature(feature.clone())?;
        
        let tol_dir = self.project_context.module_path("tol");
        self.repository.save_to_directory(&tol_dir)?;
        
        println!("✓ Feature '{}' added successfully!", feature.name);
        println!("ID: {}", feature.id);
        
        Ok(())
    }
    
    pub async fn add_stackup_interactive(&mut self) -> Result<()> {
        let name = Text::new("Stackup name:")
            .prompt()?;
        
        let description = Text::new("Description:")
            .prompt()?;
        
        let target_str = Text::new("Target dimension:")
            .with_default("0.0")
            .prompt()?;
        let target_dimension: f64 = target_str.parse().unwrap_or(0.0);
        
        let mut stackup = Stackup::new(name, description, target_dimension);
        
        // Add features to the dimension chain
        let features = self.repository.get_features();
        if !features.is_empty() {
            let add_features = Confirm::new("Add features to dimension chain?")
                .with_default(true)
                .prompt()?;
            
            if add_features {
                loop {
                    let feature_options: Vec<String> = features.iter()
                        .map(|f| format!("{} - {} ({})", f.name, f.description, f.nominal))
                        .collect();
                    
                    if feature_options.is_empty() {
                        break;
                    }
                    
                    let feature_selection = Select::new("Select feature to add:", feature_options.clone()).prompt()?;
                    let feature_index = feature_options.iter().position(|x| x == &feature_selection).unwrap();
                    let selected_feature = &features[feature_index];
                    
                    stackup.add_dimension(selected_feature.id);
                    println!("Added feature: {}", selected_feature.name);
                    
                    let continue_adding = Confirm::new("Add another feature?")
                        .with_default(false)
                        .prompt()?;
                    
                    if !continue_adding {
                        break;
                    }
                }
            }
        }
        
        self.repository.add_stackup(stackup.clone())?;
        
        let tol_dir = self.project_context.module_path("tol");
        self.repository.save_to_directory(&tol_dir)?;
        
        println!("✓ Stackup '{}' added successfully!", stackup.name);
        println!("ID: {}", stackup.id);
        println!("Features in chain: {}", stackup.dimension_chain.len());
        
        Ok(())
    }
    
    pub async fn run_analysis_interactive(&mut self) -> Result<()> {
        let stackups = self.repository.get_stackups();
        if stackups.is_empty() {
            println!("No stackups found. Create stackups first.");
            return Ok(());
        }
        
        // Select stackup to analyze
        let stackup_options: Vec<String> = stackups.iter()
            .map(|s| format!("{} - {} ({} features)", s.name, s.description, s.dimension_chain.len()))
            .collect();
        
        let stackup_selection = Select::new("Select stackup to analyze:", stackup_options.clone()).prompt()?;
        let stackup_index = stackup_options.iter().position(|x| x == &stackup_selection).unwrap();
        let stackup = &stackups[stackup_index];
        
        if stackup.dimension_chain.is_empty() {
            println!("Stackup has no features. Add features to the stackup first.");
            return Ok(());
        }
        
        // Select analysis method
        let analysis_methods = vec![
            "Worst Case",
            "Root Sum Square (RSS)",
            "Monte Carlo",
        ];
        
        let method_selection = Select::new("Select analysis method:", analysis_methods).prompt()?;
        let analysis_method = match method_selection {
            "Worst Case" => AnalysisMethod::WorstCase,
            "Root Sum Square (RSS)" => AnalysisMethod::RootSumSquare,
            "Monte Carlo" => AnalysisMethod::MonteCarlo,
            _ => AnalysisMethod::MonteCarlo,
        };
        
        // Configure Monte Carlo if selected
        let mut simulations = 10000;
        let mut confidence_level = 0.95;
        
        if let AnalysisMethod::MonteCarlo = analysis_method {
            let sim_str = Text::new("Number of simulations:")
                .with_default("10000")
                .prompt()?;
            simulations = sim_str.parse().unwrap_or(10000);
            
            let conf_str = Text::new("Confidence level (0.0-1.0):")
                .with_default("0.95")
                .prompt()?;
            confidence_level = conf_str.parse().unwrap_or(0.95);
        }
        
        let config = AnalysisConfig {
            method: analysis_method,
            simulations,
            confidence_level,
        };
        
        // Configure feature contributions
        let features = self.repository.get_features();
        let stackup_features: Vec<_> = stackup.dimension_chain.iter()
            .filter_map(|&id| features.iter().find(|f| f.id == id))
            .collect();
        
        let mut feature_contributions = Vec::new();
        
        println!("\nConfiguring feature contributions:");
        for feature in &stackup_features {
            println!("\nFeature: {} ({:.3})", feature.name, feature.nominal);
            
            let direction_options = vec![
                "Additive (+1.0)",
                "Subtractive (-1.0)",
                "Custom multiplier",
            ];
            
            let direction_selection = Select::new("Direction/multiplier:", direction_options).prompt()?;
            
            let (direction, contribution_type) = match direction_selection {
                "Additive (+1.0)" => (1.0, ContributionType::Additive),
                "Subtractive (-1.0)" => (-1.0, ContributionType::Subtractive),
                "Custom multiplier" => {
                    let custom_str = Text::new("Enter custom multiplier:")
                        .with_default("1.0")
                        .prompt()?;
                    let custom: f64 = custom_str.parse().unwrap_or(1.0);
                    (custom, ContributionType::Custom(custom))
                },
                _ => (1.0, ContributionType::Additive),
            };
            
            let half_count = Confirm::new("Half contribution?")
                .with_default(false)
                .prompt()?;
            
            feature_contributions.push(FeatureContribution {
                feature_id: feature.id,
                feature_name: feature.name.clone(),
                direction,
                half_count,
                contribution_type,
            });
        }
        
        // Run the analysis
        println!("\nRunning tolerance analysis for stackup: {}", stackup.name);
        let analyzer = ToleranceAnalyzer::new(config.clone());
        let mut analysis = analyzer.analyze_stackup_with_contributions(stackup, features, &feature_contributions)?;
        analysis.feature_contributions = feature_contributions;
        
        self.repository.add_analysis(analysis.clone())?;
        
        let tol_dir = self.project_context.module_path("tol");
        self.repository.save_to_directory(&tol_dir)?;
        
        // Display results
        self.display_analysis_results(&analysis);
        
        Ok(())
    }
    
    pub async fn configure_analysis_interactive(&mut self) -> Result<()> {
        println!("Analysis configuration options:");
        println!("1. Default configurations are applied during analysis");
        println!("2. Monte Carlo: 10,000 simulations, 95% confidence");
        println!("3. Feature contributions: Additive by default");
        println!("4. Use 'Run Analysis' to configure specific analysis settings");
        
        Ok(())
    }
    
    fn display_analysis_results(&self, analysis: &StackupAnalysis) {
        println!("\n{}", "Analysis Results".to_uppercase());
        println!("{}", "=".repeat(50));
        println!("Method: {:?}", analysis.config.method);
        println!("Stackup: {}", analysis.stackup_name);
        println!("Generated: {}", analysis.created.format("%Y-%m-%d %H:%M:%S"));
        
        println!("\n{}", "Stackup Statistics");
        println!("{}", "-".repeat(30));
        println!("Nominal Dimension: {:.6}", analysis.results.nominal_dimension);
        println!("Predicted Tolerance: +{:.6} / -{:.6}", 
                 analysis.results.predicted_tolerance.plus, 
                 analysis.results.predicted_tolerance.minus);
        
        println!("\n{}", "Process Capability");
        println!("{}", "-".repeat(30));
        println!("Cp (Process Capability): {:.3}", analysis.results.cp);
        println!("Cpk (Process Capability Index): {:.3}", analysis.results.cpk);
        println!("Sigma Level: {:.2}", analysis.results.sigma_level);
        println!("Yield Percentage: {:.2}%", analysis.results.yield_percentage);
        
        if !analysis.feature_contributions.is_empty() {
            println!("\n{}", "Feature Contributions");
            println!("{}", "-".repeat(50));
            for (i, contrib) in analysis.feature_contributions.iter().enumerate() {
                let half_indicator = if contrib.half_count { " (Half)" } else { "" };
                println!("{}. {} - Direction: {:.2}{}",
                         i + 1, contrib.feature_name, contrib.direction, half_indicator);
            }
        }
        
        if let Some(ref dist_data) = analysis.results.distribution_data {
            if !dist_data.is_empty() {
                println!("\n{}", "Distribution Data");
                println!("{}", "-".repeat(30));
                println!("Samples: {}", dist_data.len());
                println!("Min: {:.6}", dist_data.iter().fold(f64::INFINITY, |a, &b| a.min(b)));
                println!("Max: {:.6}", dist_data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)));
            }
        }
    }
    
    pub fn run_analysis(&mut self, stackup_id: Option<Id>) -> Result<()> {
        let stackups = self.repository.get_stackups();
        if stackups.is_empty() {
            println!("No stackups found. Create stackups first.");
            return Ok(());
        }
        
        let target_stackup = if let Some(id) = stackup_id {
            stackups.iter().find(|s| s.id == id)
        } else {
            // If no specific stackup, analyze the first one
            stackups.first()
        };
        
        let stackup = match target_stackup {
            Some(s) => s,
            None => {
                println!("Stackup not found.");
                return Ok(());
            }
        };
        
        let features = self.repository.get_features();
        let analyzer = ToleranceAnalyzer::default();
        
        println!("Running tolerance analysis for stackup: {}", stackup.name);
        
        // Create default feature contributions
        let feature_contributions: Vec<FeatureContribution> = stackup.dimension_chain.iter()
            .filter_map(|&id| features.iter().find(|f| f.id == id))
            .map(|f| FeatureContribution {
                feature_id: f.id,
                feature_name: f.name.clone(),
                direction: 1.0,
                half_count: false,
                contribution_type: ContributionType::Additive,
            })
            .collect();
        
        let mut analysis = analyzer.analyze_stackup_with_contributions(stackup, features, &feature_contributions)?;
        analysis.feature_contributions = feature_contributions;
        
        self.repository.add_analysis(analysis.clone())?;
        
        let tol_dir = self.project_context.module_path("tol");
        self.repository.save_to_directory(&tol_dir)?;
        
        self.display_analysis_results(&analysis);
        
        Ok(())
    }
    
    pub async fn add_mate_interactive(&mut self) -> Result<()> {
        let features = self.repository.get_features();
        if features.len() < 2 {
            println!("Need at least 2 features to create a mate.");
            return Ok(());
        }
        
        let name = Text::new("Mate name:")
            .prompt()?;
        
        let description = Text::new("Description:")
            .prompt()?;
        
        let feature_options: Vec<String> = features.iter()
            .map(|f| format!("{} - {} ({})", f.name, f.description, f.nominal))
            .collect();
        
        let primary_selection = Select::new("Select primary feature:", feature_options.clone()).prompt()?;
        let primary_index = feature_options.iter().position(|x| x == &primary_selection).unwrap();
        let primary_feature = &features[primary_index];
        
        let secondary_selection = Select::new("Select secondary feature:", feature_options.clone()).prompt()?;
        let secondary_index = feature_options.iter().position(|x| x == &secondary_selection).unwrap();
        let secondary_feature = &features[secondary_index];
        
        if primary_feature.id == secondary_feature.id {
            println!("Primary and secondary features cannot be the same.");
            return Ok(());
        }
        
        let mate_types = vec![
            "Clearance",
            "Transition",
            "Interference",
        ];
        
        let mate_type_str = Select::new("Mate type:", mate_types).prompt()?;
        let mate_type = match mate_type_str {
            "Clearance" => MateType::Clearance,
            "Transition" => MateType::Transition,
            "Interference" => MateType::Interference,
            _ => MateType::Clearance,
        };
        
        let mut mate = Mate::new(name, description, mate_type, primary_feature.id, secondary_feature.id);
        
        let offset_str = Text::new("Offset (optional):")
            .with_default("0.0")
            .prompt()?;
        mate.offset = offset_str.parse().unwrap_or(0.0);
        
        // Validate the mate
        let validation = mate.validate_fit(primary_feature, secondary_feature);
        if !validation.is_valid {
            println!("⚠️  Mate validation warning: {}", validation.error_message.unwrap_or("Unknown issue".to_string()));
            let continue_anyway = Confirm::new("Continue anyway?")
                .with_default(false)
                .prompt()?;
            if !continue_anyway {
                return Ok(());
            }
        } else {
            println!("✓ Mate validation passed");
            println!("  Nominal fit: {:.6}", validation.nominal_fit);
            println!("  Min fit: {:.6}", validation.min_fit);
            println!("  Max fit: {:.6}", validation.max_fit);
        }
        
        self.repository.add_mate(mate.clone())?;
        
        let tol_dir = self.project_context.module_path("tol");
        self.repository.save_to_directory(&tol_dir)?;
        
        println!("✓ Mate '{}' added successfully!", mate.name);
        println!("ID: {}", mate.id);
        
        Ok(())
    }
    
    pub fn list_mates(&self) -> Result<()> {
        let mates = self.repository.get_mates();
        let features = self.repository.get_features();
        
        if mates.is_empty() {
            println!("No mates found");
            return Ok(());
        }
        
        println!("Mates:");
        for (i, mate) in mates.iter().enumerate() {
            let primary_feature = features.iter().find(|f| f.id == mate.primary_feature);
            let secondary_feature = features.iter().find(|f| f.id == mate.secondary_feature);
            
            println!("{}. {} - {} ({})", i + 1, mate.name, mate.description, mate.mate_type);
            if let (Some(pf), Some(sf)) = (primary_feature, secondary_feature) {
                println!("   Primary: {} ({:.3}) - {} MMC: {:.6}, LMC: {:.6}", 
                         pf.name, pf.nominal, pf.feature_category, pf.mmc(), pf.lmc());
                println!("   Secondary: {} ({:.3}) - {} MMC: {:.6}, LMC: {:.6}", 
                         sf.name, sf.nominal, sf.feature_category, sf.mmc(), sf.lmc());
                
                let validation = mate.validate_fit(pf, sf);
                let mmc_fit = mate.calculate_mmc_fit(pf, sf);
                let lmc_fit = mate.calculate_lmc_fit(pf, sf);
                
                if validation.is_valid {
                    println!("   ✓ Valid fit:");
                    println!("     Nominal: {:.6}", validation.nominal_fit);
                    println!("     MMC (tightest): {:.6}", mmc_fit);
                    println!("     LMC (loosest): {:.6}", lmc_fit);
                    println!("     Range: {:.6} to {:.6}", validation.min_fit, validation.max_fit);
                } else {
                    println!("   ❌ Invalid fit: {}", validation.error_message.unwrap_or("Unknown issue".to_string()));
                    println!("     MMC condition: {:.6}", mmc_fit);
                    println!("     LMC condition: {:.6}", lmc_fit);
                }
            }
            println!("   ID: {}", mate.id);
            println!();
        }
        
        Ok(())
    }
    
    pub async fn edit_component_interactive(&mut self) -> Result<()> {
        let components = self.repository.get_components();
        if components.is_empty() {
            println!("No components found. Add components first.");
            return Ok(());
        }
        
        let component_options: Vec<String> = components.iter()
            .map(|c| format!("{} - {}", c.name, c.description))
            .collect();
        
        let component_selection = Select::new("Select component to edit:", component_options.clone()).prompt()?;
        let component_index = component_options.iter().position(|x| x == &component_selection).unwrap();
        let mut component = components[component_index].clone();
        
        println!("Editing component: {}", component.name);
        
        let edit_options = vec![
            "Name",
            "Description", 
            "Part Number",
            "Done",
        ];
        
        loop {
            let edit_selection = Select::new("What would you like to edit?", edit_options.clone()).prompt()?;
            
            match edit_selection {
                "Name" => {
                    let new_name = Text::new("New name:")
                        .with_default(&component.name)
                        .prompt()?;
                    if !new_name.is_empty() {
                        component.name = new_name;
                        component.updated = chrono::Utc::now();
                    }
                },
                "Description" => {
                    let new_description = Text::new("New description:")
                        .with_default(&component.description)
                        .prompt()?;
                    if !new_description.is_empty() {
                        component.description = new_description;
                        component.updated = chrono::Utc::now();
                    }
                },
                "Part Number" => {
                    let current_part_number = component.part_number.as_deref().unwrap_or("");
                    let new_part_number = Text::new("New part number (leave blank to remove):")
                        .with_default(current_part_number)
                        .prompt()?;
                    
                    if new_part_number.is_empty() {
                        component.part_number = None;
                    } else {
                        component.part_number = Some(new_part_number);
                    }
                    component.updated = chrono::Utc::now();
                },
                "Done" => break,
                _ => {}
            }
        }
        
        // Update the component in the repository
        self.repository.update_component(component.clone())?;
        
        let tol_dir = self.project_context.module_path("tol");
        self.repository.save_to_directory(&tol_dir)?;
        
        println!("✓ Component '{}' updated successfully!", component.name);
        
        Ok(())
    }
    
    pub async fn edit_feature_interactive(&mut self) -> Result<()> {
        let features = self.repository.get_features();
        if features.is_empty() {
            println!("No features found. Add features first.");
            return Ok(());
        }
        
        let feature_options: Vec<String> = features.iter()
            .map(|f| format!("{} - {} ({})", f.name, f.description, f.nominal))
            .collect();
        
        let feature_selection = Select::new("Select feature to edit:", feature_options.clone()).prompt()?;
        let feature_index = feature_options.iter().position(|x| x == &feature_selection).unwrap();
        let mut feature = features[feature_index].clone();
        
        println!("Editing feature: {}", feature.name);
        
        let edit_options = vec![
            "Name",
            "Description",
            "Nominal Value",
            "Plus Tolerance",
            "Minus Tolerance",
            "Distribution Type",
            "Drawing Location",
            "Done",
        ];
        
        loop {
            let edit_selection = Select::new("What would you like to edit?", edit_options.clone()).prompt()?;
            
            match edit_selection {
                "Name" => {
                    let new_name = Text::new("New name:")
                        .with_default(&feature.name)
                        .prompt()?;
                    if !new_name.is_empty() {
                        feature.name = new_name;
                        feature.updated = chrono::Utc::now();
                    }
                },
                "Description" => {
                    let new_description = Text::new("New description:")
                        .with_default(&feature.description)
                        .prompt()?;
                    if !new_description.is_empty() {
                        feature.description = new_description;
                        feature.updated = chrono::Utc::now();
                    }
                },
                "Nominal Value" => {
                    let nominal_str = Text::new("New nominal value:")
                        .with_default(&feature.nominal.to_string())
                        .prompt()?;
                    if let Ok(nominal) = nominal_str.parse::<f64>() {
                        feature.nominal = nominal;
                        feature.updated = chrono::Utc::now();
                        feature.distribution_params = Some(DistributionParams::calculate_from_feature(&feature));
                    }
                },
                "Plus Tolerance" => {
                    let plus_str = Text::new("New plus tolerance:")
                        .with_default(&feature.tolerance.plus.to_string())
                        .prompt()?;
                    if let Ok(plus) = plus_str.parse::<f64>() {
                        feature.tolerance.plus = plus;
                        feature.updated = chrono::Utc::now();
                        feature.distribution_params = Some(DistributionParams::calculate_from_feature(&feature));
                    }
                },
                "Minus Tolerance" => {
                    let minus_str = Text::new("New minus tolerance:")
                        .with_default(&feature.tolerance.minus.to_string())
                        .prompt()?;
                    if let Ok(minus) = minus_str.parse::<f64>() {
                        feature.tolerance.minus = minus;
                        feature.updated = chrono::Utc::now();
                        feature.distribution_params = Some(DistributionParams::calculate_from_feature(&feature));
                    }
                },
                "Distribution Type" => {
                    let distribution_types = vec![
                        "Normal",
                        "Uniform",
                        "Triangular",
                        "LogNormal",
                        "Beta",
                    ];
                    
                    let current_dist = match feature.distribution.as_ref().unwrap_or(&ToleranceDistribution::Normal) {
                        ToleranceDistribution::Normal => "Normal",
                        ToleranceDistribution::Uniform => "Uniform",
                        ToleranceDistribution::Triangular => "Triangular",
                        ToleranceDistribution::LogNormal => "LogNormal",
                        ToleranceDistribution::Beta { .. } => "Beta",
                    };
                    
                    let distribution_str = Select::new("Distribution type:", distribution_types)
                        .with_help_message(&format!("Current: {}", current_dist))
                        .prompt()?;
                    
                    let distribution_type = match distribution_str {
                        "Normal" => ToleranceDistribution::Normal,
                        "Uniform" => ToleranceDistribution::Uniform,
                        "Triangular" => ToleranceDistribution::Triangular,
                        "LogNormal" => ToleranceDistribution::LogNormal,
                        "Beta" => {
                            let alpha_str = Text::new("Beta alpha parameter:")
                                .with_default("2.0")
                                .prompt()?;
                            let alpha: f64 = alpha_str.parse().unwrap_or(2.0);
                            
                            let beta_str = Text::new("Beta beta parameter:")
                                .with_default("2.0")
                                .prompt()?;
                            let beta: f64 = beta_str.parse().unwrap_or(2.0);
                            
                            ToleranceDistribution::Beta { alpha, beta }
                        }
                        _ => ToleranceDistribution::Normal,
                    };
                    
                    feature.update_distribution(distribution_type);
                },
                "Drawing Location" => {
                    let current_location = feature.drawing_location.as_deref().unwrap_or("");
                    let new_location = Text::new("New drawing location (leave blank to remove):")
                        .with_default(current_location)
                        .prompt()?;
                    
                    if new_location.is_empty() {
                        feature.drawing_location = None;
                    } else {
                        feature.drawing_location = Some(new_location);
                    }
                    feature.updated = chrono::Utc::now();
                },
                "Done" => break,
                _ => {}
            }
        }
        
        // Update the feature in the repository
        self.repository.update_feature(feature.clone())?;
        
        let tol_dir = self.project_context.module_path("tol");
        self.repository.save_to_directory(&tol_dir)?;
        
        println!("✓ Feature '{}' updated successfully!", feature.name);
        
        Ok(())
    }
    
    pub async fn edit_mate_interactive(&mut self) -> Result<()> {
        let mates = self.repository.get_mates();
        if mates.is_empty() {
            println!("No mates found. Add mates first.");
            return Ok(());
        }
        
        let features = self.repository.get_features();
        let mate_options: Vec<String> = mates.iter()
            .map(|m| format!("{} - {} ({})", m.name, m.description, m.mate_type))
            .collect();
        
        let mate_selection = Select::new("Select mate to edit:", mate_options.clone()).prompt()?;
        let mate_index = mate_options.iter().position(|x| x == &mate_selection).unwrap();
        let mut mate = mates[mate_index].clone();
        
        println!("Editing mate: {}", mate.name);
        
        let edit_options = vec![
            "Name",
            "Description",
            "Mate Type",
            "Primary Feature",
            "Secondary Feature",
            "Offset",
            "Done",
        ];
        
        loop {
            let edit_selection = Select::new("What would you like to edit?", edit_options.clone()).prompt()?;
            
            match edit_selection {
                "Name" => {
                    let new_name = Text::new("New name:")
                        .with_default(&mate.name)
                        .prompt()?;
                    if !new_name.is_empty() {
                        mate.name = new_name;
                        mate.updated = chrono::Utc::now();
                    }
                },
                "Description" => {
                    let new_description = Text::new("New description:")
                        .with_default(&mate.description)
                        .prompt()?;
                    if !new_description.is_empty() {
                        mate.description = new_description;
                        mate.updated = chrono::Utc::now();
                    }
                },
                "Mate Type" => {
                    let mate_types = vec![
                        "Clearance",
                        "Transition",
                        "Interference",
                    ];
                    
                    let current_type = format!("{}", mate.mate_type);
                    let mate_type_str = Select::new("Mate type:", mate_types)
                        .with_help_message(&format!("Current: {}", current_type))
                        .prompt()?;
                    
                    let mate_type = match mate_type_str {
                        "Clearance" => MateType::Clearance,
                        "Transition" => MateType::Transition,
                        "Interference" => MateType::Interference,
                        _ => MateType::Clearance,
                    };
                    
                    mate.mate_type = mate_type;
                    mate.updated = chrono::Utc::now();
                },
                "Primary Feature" | "Secondary Feature" => {
                    let feature_options: Vec<String> = features.iter()
                        .map(|f| format!("{} - {} ({})", f.name, f.description, f.nominal))
                        .collect();
                    
                    if feature_options.is_empty() {
                        println!("No features available for selection.");
                        continue;
                    }
                    
                    let feature_selection = Select::new(&format!("Select new {}:", edit_selection.to_lowercase()), feature_options.clone()).prompt()?;
                    let feature_index = feature_options.iter().position(|x| x == &feature_selection).unwrap();
                    let selected_feature = &features[feature_index];
                    
                    if edit_selection == "Primary Feature" {
                        if selected_feature.id == mate.secondary_feature {
                            println!("Primary and secondary features cannot be the same.");
                            continue;
                        }
                        mate.primary_feature = selected_feature.id;
                    } else {
                        if selected_feature.id == mate.primary_feature {
                            println!("Primary and secondary features cannot be the same.");
                            continue;
                        }
                        mate.secondary_feature = selected_feature.id;
                    }
                    mate.updated = chrono::Utc::now();
                },
                "Offset" => {
                    let offset_str = Text::new("New offset:")
                        .with_default(&mate.offset.to_string())
                        .prompt()?;
                    if let Ok(offset) = offset_str.parse::<f64>() {
                        mate.offset = offset;
                        mate.updated = chrono::Utc::now();
                    }
                },
                "Done" => break,
                _ => {}
            }
        }
        
        // Validate the updated mate
        let primary_feature = features.iter().find(|f| f.id == mate.primary_feature);
        let secondary_feature = features.iter().find(|f| f.id == mate.secondary_feature);
        
        if let (Some(pf), Some(sf)) = (primary_feature, secondary_feature) {
            let validation = mate.validate_fit(pf, sf);
            if !validation.is_valid {
                println!("⚠️  Mate validation warning: {}", validation.error_message.unwrap_or("Unknown issue".to_string()));
                let continue_anyway = Confirm::new("Continue anyway?")
                    .with_default(false)
                    .prompt()?;
                if !continue_anyway {
                    return Ok(());
                }
            } else {
                println!("✓ Mate validation passed");
                println!("  Nominal fit: {:.6}", validation.nominal_fit);
                println!("  Min fit: {:.6}", validation.min_fit);
                println!("  Max fit: {:.6}", validation.max_fit);
            }
        }
        
        // Update the mate in the repository
        self.repository.update_mate(mate.clone())?;
        
        let tol_dir = self.project_context.module_path("tol");
        self.repository.save_to_directory(&tol_dir)?;
        
        println!("✓ Mate '{}' updated successfully!", mate.name);
        
        Ok(())
    }
    
    pub async fn edit_stackup_interactive(&mut self) -> Result<()> {
        let stackups = self.repository.get_stackups();
        if stackups.is_empty() {
            println!("No stackups found. Add stackups first.");
            return Ok(());
        }
        
        let stackup_options: Vec<String> = stackups.iter()
            .map(|s| format!("{} - {} ({} features)", s.name, s.description, s.dimension_chain.len()))
            .collect();
        
        let stackup_selection = Select::new("Select stackup to edit:", stackup_options.clone()).prompt()?;
        let stackup_index = stackup_options.iter().position(|x| x == &stackup_selection).unwrap();
        let mut stackup = stackups[stackup_index].clone();
        
        println!("Editing stackup: {}", stackup.name);
        
        let edit_options = vec![
            "Name",
            "Description",
            "Target Dimension",
            "Tolerance Target",
            "Manage Features",
            "Done",
        ];
        
        loop {
            let edit_selection = Select::new("What would you like to edit?", edit_options.clone()).prompt()?;
            
            match edit_selection {
                "Name" => {
                    let new_name = Text::new("New name:")
                        .with_default(&stackup.name)
                        .prompt()?;
                    if !new_name.is_empty() {
                        stackup.name = new_name;
                        stackup.updated = chrono::Utc::now();
                    }
                },
                "Description" => {
                    let new_description = Text::new("New description:")
                        .with_default(&stackup.description)
                        .prompt()?;
                    if !new_description.is_empty() {
                        stackup.description = new_description;
                        stackup.updated = chrono::Utc::now();
                    }
                },
                "Target Dimension" => {
                    let target_str = Text::new("New target dimension:")
                        .with_default(&stackup.target_dimension.to_string())
                        .prompt()?;
                    if let Ok(target) = target_str.parse::<f64>() {
                        stackup.target_dimension = target;
                        stackup.updated = chrono::Utc::now();
                    }
                },
                "Tolerance Target" => {
                    let plus_str = Text::new("Plus tolerance:")
                        .with_default(&stackup.tolerance_target.plus.to_string())
                        .prompt()?;
                    let plus: f64 = plus_str.parse().unwrap_or(stackup.tolerance_target.plus);
                    
                    let minus_str = Text::new("Minus tolerance:")
                        .with_default(&stackup.tolerance_target.minus.to_string())
                        .prompt()?;
                    let minus: f64 = minus_str.parse().unwrap_or(stackup.tolerance_target.minus);
                    
                    stackup.tolerance_target.plus = plus;
                    stackup.tolerance_target.minus = minus;
                    stackup.updated = chrono::Utc::now();
                },
                "Manage Features" => {
                    // Submenu for managing features in the stackup
                    self.manage_stackup_features(&mut stackup).await?;
                },
                "Done" => break,
                _ => {}
            }
        }
        
        // Update the stackup in the repository
        self.repository.update_stackup(stackup.clone())?;
        
        let tol_dir = self.project_context.module_path("tol");
        self.repository.save_to_directory(&tol_dir)?;
        
        println!("✓ Stackup '{}' updated successfully!", stackup.name);
        
        Ok(())
    }
    
    async fn manage_stackup_features(&mut self, stackup: &mut Stackup) -> Result<()> {
        let features = self.repository.get_features();
        
        loop {
            let manage_options = vec![
                "Add Feature",
                "Remove Feature", 
                "List Current Features",
                "Done",
            ];
            
            let manage_selection = Select::new("Manage features:", manage_options).prompt()?;
            
            match manage_selection {
                "Add Feature" => {
                    let available_features: Vec<_> = features.iter()
                        .filter(|f| !stackup.dimension_chain.contains(&f.id))
                        .collect();
                    
                    if available_features.is_empty() {
                        println!("No available features to add.");
                        continue;
                    }
                    
                    let feature_options: Vec<String> = available_features.iter()
                        .map(|f| format!("{} - {} ({})", f.name, f.description, f.nominal))
                        .collect();
                    
                    let feature_selection = Select::new("Select feature to add:", feature_options.clone()).prompt()?;
                    let feature_index = feature_options.iter().position(|x| x == &feature_selection).unwrap();
                    let selected_feature = available_features[feature_index];
                    
                    stackup.add_dimension(selected_feature.id);
                    println!("Added feature: {}", selected_feature.name);
                },
                "Remove Feature" => {
                    if stackup.dimension_chain.is_empty() {
                        println!("No features in stackup to remove.");
                        continue;
                    }
                    
                    let current_features: Vec<_> = stackup.dimension_chain.iter()
                        .filter_map(|&id| features.iter().find(|f| f.id == id))
                        .collect();
                    
                    let feature_options: Vec<String> = current_features.iter()
                        .map(|f| format!("{} - {} ({})", f.name, f.description, f.nominal))
                        .collect();
                    
                    let feature_selection = Select::new("Select feature to remove:", feature_options.clone()).prompt()?;
                    let feature_index = feature_options.iter().position(|x| x == &feature_selection).unwrap();
                    let selected_feature = current_features[feature_index];
                    
                    stackup.dimension_chain.retain(|&id| id != selected_feature.id);
                    stackup.updated = chrono::Utc::now();
                    println!("Removed feature: {}", selected_feature.name);
                },
                "List Current Features" => {
                    if stackup.dimension_chain.is_empty() {
                        println!("No features in stackup.");
                    } else {
                        println!("Current features in stackup:");
                        for (i, &feature_id) in stackup.dimension_chain.iter().enumerate() {
                            if let Some(feature) = features.iter().find(|f| f.id == feature_id) {
                                println!("  {}. {} ({:.3})", i + 1, feature.name, feature.nominal);
                            }
                        }
                    }
                },
                "Done" => break,
                _ => {}
            }
        }
        
        Ok(())
    }
    
    pub fn list_components(&self) -> Result<()> {
        let components = self.repository.get_components();
        
        if components.is_empty() {
            println!("No components found");
            return Ok(());
        }
        
        println!("Components:");
        for (i, component) in components.iter().enumerate() {
            println!("{}. {} - {}", i + 1, component.name, component.description);
            if let Some(ref pn) = component.part_number {
                println!("   Part Number: {}", pn);
            }
            println!("   ID: {}", component.id);
            
            let features = self.repository.get_features_for_component(component.id);
            println!("   Features: {}", features.len());
            for feature in features {
                let distribution_info = if let Some(ref dist) = feature.distribution {
                    format!(" [{}]", match dist {
                        ToleranceDistribution::Normal => "Normal",
                        ToleranceDistribution::Uniform => "Uniform", 
                        ToleranceDistribution::Triangular => "Triangular",
                        ToleranceDistribution::LogNormal => "LogNormal",
                        ToleranceDistribution::Beta { .. } => "Beta",
                    })
                } else {
                    String::new()
                };
                
                let drawing_location = feature.drawing_location
                    .as_ref()
                    .map(|loc| format!(" @{}", loc))
                    .unwrap_or_default();
                
                println!("     - {} ({:.3} +{:.3}/-{:.3}){}{}",
                         feature.name, feature.nominal, feature.tolerance.plus, feature.tolerance.minus,
                         distribution_info, drawing_location);
            }
            println!();
        }
        
        Ok(())
    }
    
    pub fn show_dashboard(&self) -> Result<()> {
        let components = self.repository.get_components();
        let features = self.repository.get_features();
        let mates = self.repository.get_mates();
        let stackups = self.repository.get_stackups();
        let analyses = self.repository.get_analyses();
        
        println!("Tolerance Analysis Dashboard");
        println!("===========================");
        println!("Components: {}", components.len());
        println!("Features: {}", features.len());
        println!("Mates: {}", mates.len());
        println!("Stackups: {}", stackups.len());
        println!("Analyses: {}", analyses.len());
        
        if !mates.is_empty() {
            println!("\nMate Analysis Summary:");
            let mut valid_mates = 0;
            let mut invalid_mates = 0;
            let mut clearance_mates = 0;
            let mut transition_mates = 0;
            let mut interference_mates = 0;
            
            for mate in mates {
                let primary_feature = features.iter().find(|f| f.id == mate.primary_feature);
                let secondary_feature = features.iter().find(|f| f.id == mate.secondary_feature);
                
                if let (Some(pf), Some(sf)) = (primary_feature, secondary_feature) {
                    let validation = mate.validate_fit(pf, sf);
                    if validation.is_valid {
                        valid_mates += 1;
                    } else {
                        invalid_mates += 1;
                    }
                    
                    match mate.mate_type {
                        MateType::Clearance => clearance_mates += 1,
                        MateType::Transition => transition_mates += 1,
                        MateType::Interference => interference_mates += 1,
                    }
                }
            }
            
            println!("  Valid mates: {} | Invalid mates: {}", valid_mates, invalid_mates);
            println!("  Clearance: {} | Transition: {} | Interference: {}", 
                     clearance_mates, transition_mates, interference_mates);
        }
        
        if !components.is_empty() {
            println!("\nComponent Interaction Summary:");
            for component in components {
                let component_features = self.repository.get_features_for_component(component.id);
                let mut interactions = 0;
                
                for feature in &component_features {
                    let feature_mates = mates.iter()
                        .filter(|m| m.primary_feature == feature.id || m.secondary_feature == feature.id)
                        .count();
                    interactions += feature_mates;
                }
                
                println!("  {} - {} features, {} interactions", 
                         component.name, component_features.len(), interactions);
            }
        }
        
        if !stackups.is_empty() {
            println!("\nStackup Summary:");
            for stackup in stackups {
                let chain_features: Vec<_> = stackup.dimension_chain
                    .iter()
                    .filter_map(|&id| features.iter().find(|f| f.id == id))
                    .collect();
                
                println!("  {} - {} features in chain", stackup.name, chain_features.len());
                
                let stackup_analyses = self.repository.get_analyses_for_stackup(stackup.id);
                if !stackup_analyses.is_empty() {
                    let latest = stackup_analyses.last().unwrap();
                    println!("    Latest Analysis: Cp={:.2}, Yield={:.1}%", 
                             latest.results.cp, latest.results.yield_percentage);
                }
            }
        }
        
        Ok(())
    }
}