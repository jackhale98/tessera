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
            _ => {
                let other_name = Text::new("Other feature type:").prompt()?;
                FeatureType::Other(other_name)
            }
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
        
        let mut feature = Feature::new(name, description, selected_component.id, feature_type, nominal);
        feature.tolerance.plus = plus_tolerance;
        feature.tolerance.minus = minus_tolerance;
        
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
        let analysis = analyzer.analyze_stackup(stackup, features)?;
        
        self.repository.add_analysis(analysis.clone())?;
        
        let tol_dir = self.project_context.module_path("tol");
        self.repository.save_to_directory(&tol_dir)?;
        
        println!("\nAnalysis Results:");
        println!("================");
        println!("Nominal Dimension: {:.6}", analysis.results.nominal_dimension);
        println!("Predicted Tolerance: +{:.6} / -{:.6}", 
                 analysis.results.predicted_tolerance.plus, 
                 analysis.results.predicted_tolerance.minus);
        println!("Process Capability (Cp): {:.3}", analysis.results.cp);
        println!("Process Capability Index (Cpk): {:.3}", analysis.results.cpk);
        println!("Sigma Level: {:.2}", analysis.results.sigma_level);
        println!("Yield Percentage: {:.2}%", analysis.results.yield_percentage);
        
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
                println!("     - {} ({:.3} +{:.3}/-{:.3})", 
                         feature.name, feature.nominal, feature.tolerance.plus, feature.tolerance.minus);
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