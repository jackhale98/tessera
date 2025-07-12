use crate::data::*;
use crate::repository::ToleranceRepository;
use crate::analysis::ToleranceAnalyzer;
use crate::sensitivity::{SensitivityAnalyzer, StackupContribution};
use tessera_core::{ProjectContext, Id, Result};
use inquire::{Select, Text, Confirm, MultiSelect};

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
        
        let mut stackup = Stackup::new(name, description);
        
        // Set engineering specification limits for process capability analysis
        let set_spec_limits = Confirm::new("Set engineering specification limits for process capability analysis?")
            .with_default(true)
            .with_help_message("Required for meaningful Cp/Cpk calculations")
            .prompt()?;
        
        if set_spec_limits {
            let usl_str = Text::new("Upper Specification Limit (USL):")
                .with_help_message("Engineering/customer maximum acceptable dimension")
                .prompt()?;
            let lsl_str = Text::new("Lower Specification Limit (LSL):")
                .with_help_message("Engineering/customer minimum acceptable dimension")
                .prompt()?;
            
            if let (Ok(usl), Ok(lsl)) = (usl_str.parse::<f64>(), lsl_str.parse::<f64>()) {
                if usl > lsl {
                    stackup.set_specification_limits(Some(usl), Some(lsl));
                    println!("✓ Specification limits set: LSL={:.6}, USL={:.6}", lsl, usl);
                    println!("✓ Target dimension calculated as midpoint: {:.6}", stackup.target_dimension);
                    println!("ℹ️  Note: Target tolerance not needed when using USL/LSL specification limits");
                } else {
                    println!("⚠ Warning: USL must be greater than LSL. Specification limits not set.");
                }
            } else {
                println!("⚠ Warning: Invalid specification limits. Process capability analysis will not be available.");
            }
        } else {
            println!("⚠ Warning: Without specification limits, process capability analysis will not be meaningful.");
        }
        
        // Add features to the dimension chain
        let add_features = {
            let features = self.repository.get_features();
            !features.is_empty() && Confirm::new("Add features to dimension chain?")
                .with_default(true)
                .prompt()?
        };
        
        if add_features {
            loop {
                // First, select component
                let components = self.repository.get_components();
                if components.is_empty() {
                    println!("No components found. Add components first.");
                    break;
                }
                
                let component_options: Vec<String> = components.iter()
                    .map(|c| format!("{} - {}", c.name, c.description))
                    .collect();
                
                let component_selection = Select::new("Select component:", component_options.clone()).prompt()?;
                let component_index = component_options.iter().position(|x| x == &component_selection).unwrap();
                let selected_component = &components[component_index];
                
                // Then, select feature from that component
                let features = self.repository.get_features();
                let component_features: Vec<&Feature> = features.iter()
                    .filter(|f| f.component_id == selected_component.id)
                    .collect();
                
                if component_features.is_empty() {
                    println!("No features found for component '{}'. Add features first.", selected_component.name);
                    break;
                }
                
                let feature_options: Vec<String> = component_features.iter()
                    .map(|f| format!("{} - {} ({})", f.name, f.description, f.nominal))
                    .collect();
                
                if feature_options.is_empty() {
                    break;
                }
                
                let feature_selection = Select::new("Select feature to add:", feature_options.clone()).prompt()?;
                let feature_index = feature_options.iter().position(|x| x == &feature_selection).unwrap();
                let selected_feature = component_features[feature_index].clone();
                
                stackup.add_dimension(selected_feature.id, selected_feature.name.clone());
                println!("Added feature: {}", selected_feature.name);
                
                // Configure vector contribution for this feature
                self.configure_feature_contribution(&mut stackup, &selected_feature).await?;
                
                let continue_adding = Confirm::new("Add another feature?")
                    .with_default(false)
                    .prompt()?;
                
                if !continue_adding {
                    break;
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
        
        // Select analysis methods using MultiSelect
        let analysis_method_options = vec![
            "Worst Case",
            "Root Sum Square (RSS)", 
            "Monte Carlo",
        ];
        
        let selected_methods = MultiSelect::new("Select analysis methods:", analysis_method_options.clone())
            .with_help_message("Use space to select/deselect, enter to confirm")
            .prompt()?;
        
        if selected_methods.is_empty() {
            println!("No analysis methods selected. Exiting analysis.");
            return Ok(());
        }
        
        // Configure Monte Carlo if selected
        let mut simulations = 10000;
        let mut confidence_level = 0.95;
        
        if selected_methods.contains(&"Monte Carlo") {
            let sim_str = Text::new("Number of simulations:")
                .with_default("10000")
                .prompt()?;
            simulations = sim_str.parse().unwrap_or(10000);
            
            let conf_str = Text::new("Confidence level (0.0-1.0):")
                .with_default("0.95")
                .prompt()?;
            confidence_level = conf_str.parse().unwrap_or(0.95);
            
            let use_seed = Confirm::new("Set random seed for reproducible results?")
                .with_default(false)
                .with_help_message("Use random seed for debugging/validation")
                .prompt()?;
            
            if use_seed {
                let seed_str = Text::new("Random seed (integer):")
                    .with_default("12345")
                    .with_help_message("Same seed produces identical results")
                    .prompt()?;
                if let Ok(seed_value) = seed_str.parse::<u64>() {
                    println!("✓ Random seed set: {}", seed_value);
                    // Note: Seed is handled by individual analyzers in their configuration
                }
            } else {
                println!("✓ Using random seed for simulation");
            }
        }
        
        // Use stackup's existing feature contributions
        let features = self.repository.get_features();
        let feature_contributions = &stackup.feature_contributions;
        
        // Run the analyses for each selected method
        println!("\nRunning tolerance analysis for stackup: {}", stackup.name);
        println!("Selected methods: {}", selected_methods.join(", "));
        
        let mut analyses = Vec::new();
        
        for method_name in &selected_methods {
            let analysis_method = match *method_name {
                "Worst Case" => AnalysisMethod::WorstCase,
                "Root Sum Square (RSS)" => AnalysisMethod::RootSumSquare,
                "Monte Carlo" => AnalysisMethod::MonteCarlo,
                _ => AnalysisMethod::MonteCarlo,
            };
            
            let config = AnalysisConfig {
                method: analysis_method,
                simulations,
                confidence_level,
                use_three_sigma: true, // Default to showing both 3-sigma and confidence intervals
            };
            
            println!("\n{}", "=".repeat(50));
            println!("Running {} Analysis", method_name);
            println!("{}", "=".repeat(50));
            
            let analyzer = ToleranceAnalyzer::new(config.clone());
            let mut analysis = analyzer.analyze_stackup_with_contributions(stackup, features, feature_contributions)?;
            analysis.feature_contributions = feature_contributions.clone();
            
            // Display results
            self.display_analysis_results(&analysis);
            
            analyses.push(analysis);
        }
        
        // Save all analyses after running them
        for analysis in analyses {
            self.repository.add_analysis(analysis)?;
        }
        
        let tol_dir = self.project_context.module_path("tol");
        self.repository.save_to_directory(&tol_dir)?;
        
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
        println!("Target Dimension: {:.6}", analysis.target_dimension);
        println!("Nominal Dimension: {:.6}", analysis.results.nominal_dimension);
        println!("Dimension Variance: {:.6}", analysis.results.nominal_dimension - analysis.target_dimension);
        
        // Show predicted tolerance based on analysis method
        let tolerance_label = match analysis.config.method {
            AnalysisMethod::WorstCase => "Worst-Case Tolerance",
            AnalysisMethod::RootSumSquare => "RSS Predicted Tolerance (3σ)",
            AnalysisMethod::MonteCarlo => "Monte Carlo Predicted Tolerance (3σ)",
        };
        println!("{}: +{:.6} / -{:.6}", 
                 tolerance_label,
                 analysis.results.predicted_tolerance.plus, 
                 analysis.results.predicted_tolerance.minus);
        
        // Show 3-sigma tolerance if available
        if let Some(ref three_sigma) = analysis.results.three_sigma_tolerance {
            println!("3-Sigma Tolerance: +{:.6} / -{:.6} (Engineering Standard)", 
                     three_sigma.plus, three_sigma.minus);
        }
        
        // Show user-specified confidence tolerance if available
        if let Some(ref user_tolerance) = analysis.results.user_specified_tolerance {
            println!("{}% Confidence Tolerance: +{:.6} / -{:.6}", 
                     (analysis.config.confidence_level * 100.0),
                     user_tolerance.plus, user_tolerance.minus);
            
            // Add statistical interpretation for Monte Carlo analyses
            if matches!(analysis.config.method, AnalysisMethod::MonteCarlo) {
                if let Some(ref three_sigma) = analysis.results.three_sigma_tolerance {
                    let ci_total = user_tolerance.plus + user_tolerance.minus;
                    let sigma3_total = three_sigma.plus + three_sigma.minus;
                    let ratio = ci_total / sigma3_total;
                    
                    println!("\n{}", "Statistical Interpretation (Monte Carlo)");
                    println!("{}", "-".repeat(50));
                    println!("Confidence Interval vs 3-Sigma Ratio: {:.3}", ratio);
                    
                    if analysis.config.confidence_level == 0.95 {
                        println!("95% CI should be ~0.653 of 3σ range for normal distributions");
                        println!("Actual ratio: {:.3} ({})", ratio, 
                                if (ratio - 0.653).abs() < 0.1 { "Expected" } else { "Check distribution assumptions" });
                    }
                    
                    println!("Distribution assumptions: Each feature tolerance = ±3σ process capability");
                    println!("Industry standard: Tolerance bands typically represent manufacturing limits");
                }
            }
        }
        
        println!("\n{}", "Process Capability Analysis");
        println!("{}", "-".repeat(50));
        
        // Get stackup to check for specification limits
        let stackups = self.repository.get_stackups();
        let stackup = stackups.iter().find(|s| s.id == analysis.stackup_id);
        
        if let Some(stackup) = stackup {
            if stackup.has_specification_limits() {
                let (usl, lsl) = stackup.get_specification_limits().unwrap();
                println!("Engineering Specification Limits:");
                println!("  Lower Spec Limit (LSL): {:.6}", lsl);
                println!("  Upper Spec Limit (USL): {:.6}", usl);
                println!("  Specification Range: {:.6}", usl - lsl);
                
                if !analysis.results.cp.is_nan() {
                    println!("\nProcess Capability Metrics:");
                    println!("  Cp (Process Capability): {:.3}", analysis.results.cp);
                    println!("  Cpk (Process Capability Index): {:.3}", analysis.results.cpk);
                    println!("  Sigma Level: {:.2}σ", analysis.results.sigma_level);
                    println!("  Yield Percentage: {:.2}%", analysis.results.yield_percentage);
                    
                    // Calculate and display PPM (parts per million defects)
                    let defect_rate = (100.0 - analysis.results.yield_percentage) / 100.0;
                    let ppm = defect_rate * 1_000_000.0;
                    println!("  Defect Rate: {:.0} PPM (parts per million)", ppm);
                    
                    // Add interpretation
                    let cp_interpretation = if analysis.results.cp >= 1.67 {
                        "Excellent (Aerospace/Medical grade)"
                    } else if analysis.results.cp >= 1.33 {
                        "Good (Automotive standard)"
                    } else if analysis.results.cp >= 1.0 {
                        "Marginal"
                    } else {
                        "Poor - Process improvement needed"
                    };
                    
                    let cpk_interpretation = if analysis.results.cpk >= 1.33 {
                        "Process is capable and centered"
                    } else if analysis.results.cpk >= 1.0 {
                        "Process marginally capable, check centering"
                    } else {
                        "Process not capable or severely off-center"
                    };
                    
                    println!("\nInterpretation:");
                    println!("  Cp Assessment: {}", cp_interpretation);
                    println!("  Cpk Assessment: {}", cpk_interpretation);
                } else {
                    println!("\n⚠ Process capability could not be calculated");
                    println!("  This may be due to insufficient data or analysis method limitations");
                }
            } else {
                println!("⚠ No engineering specification limits defined");
                println!("Process capability analysis requires LSL and USL to be meaningful.");
                println!("Current values shown are based on analysis predictions only:");
                println!("  Cp: {:.3} (Not meaningful)", analysis.results.cp);
                println!("  Cpk: {:.3} (Not meaningful)", analysis.results.cpk);
                println!("  Yield: {:.2}% (Based on predicted tolerance)", analysis.results.yield_percentage);
                println!("\n💡 Tip: Set specification limits during stackup creation for proper Cp/Cpk analysis");
            }
        }
        
        if !analysis.feature_contributions.is_empty() {
            println!("\n{}", "Feature Contributions");
            println!("{}", "-".repeat(70));
            for (i, contrib) in analysis.feature_contributions.iter().enumerate() {
                let half_indicator = if contrib.half_count { " (Half)" } else { "" };
                println!("{}. {} - Direction: {:.2}{}",
                         i + 1, contrib.feature_name, contrib.direction, half_indicator);
            }
            
            // Add sensitivity analysis
            if let Ok(sensitivity) = self.calculate_sensitivity_analysis(analysis) {
                println!("\n{}", "Sensitivity Analysis (Stackup Impact)");
                println!("{}", "-".repeat(70));
                println!("Total Variance: {:.6}", sensitivity.total_variance);
                println!("Total Standard Deviation: {:.6}", sensitivity.total_std_dev);
                
                println!("\nFeature Impact Ranking (Including Geometric Multipliers):");
                for contrib in &sensitivity.contributions {
                    println!("{}. {} - {:.2}% contribution (variance: {:.6})",
                             contrib.rank, contrib.feature_name, contrib.percentage, 
                             contrib.variance_contribution);
                }
                
                // Add tolerance-normalized sensitivity
                if let Ok(tolerance_sensitivity) = self.calculate_tolerance_normalized_sensitivity(analysis) {
                    println!("\n{}", "Tolerance-Normalized Sensitivity (Pure Tolerance Impact)");
                    println!("{}", "-".repeat(70));
                    println!("Note: This shows sensitivity based purely on tolerance magnitude,");
                    println!("ignoring geometric multipliers from stackup configuration.");
                    
                    for (i, contrib) in tolerance_sensitivity.iter().enumerate() {
                        println!("{}. {} - {:.2}% (tolerance: ±{:.6})",
                                 i + 1, contrib.0, contrib.1, contrib.2);
                    }
                }
            }
        }
        
        // Show nominal and tolerance values for each contribution
        if !analysis.feature_contributions.is_empty() {
            println!("\n{}", "Feature Details");
            println!("{}", "-".repeat(70));
            let features = self.repository.get_features();
            for contrib in &analysis.feature_contributions {
                if let Some(feature) = features.iter().find(|f| f.id == contrib.feature_id) {
                    println!("{}: Nominal = {:.6}, Tolerance = +{:.6}/-{:.6}",
                             feature.name, feature.nominal, feature.tolerance.plus, feature.tolerance.minus);
                }
            }
        }
        
        // Show quartile data if available (Monte Carlo only)
        if let Some(ref quartiles) = analysis.results.quartile_data {
            println!("\n{}", "Distribution Analysis");
            println!("{}", "-".repeat(50));
            println!("Minimum: {:.6}", quartiles.minimum);
            println!("5th Percentile: {:.6}", quartiles.p5);
            println!("1st Quartile (Q1): {:.6}", quartiles.q1);
            println!("Median (Q2): {:.6}", quartiles.median);
            println!("3rd Quartile (Q3): {:.6}", quartiles.q3);
            println!("95th Percentile: {:.6}", quartiles.p95);
            println!("Maximum: {:.6}", quartiles.maximum);
            println!("Interquartile Range (IQR): {:.6}", quartiles.iqr);
            
            // Box plot-style visualization
            let range = quartiles.maximum - quartiles.minimum;
            if range > 0.0 {
                println!("\nBox Plot Visualization:");
                let q1_pos = ((quartiles.q1 - quartiles.minimum) / range * 50.0) as usize;
                let median_pos = ((quartiles.median - quartiles.minimum) / range * 50.0) as usize;
                let q3_pos = ((quartiles.q3 - quartiles.minimum) / range * 50.0) as usize;
                
                let mut plot = vec!['-'; 52];
                plot[0] = '|';  // Min
                plot[51] = '|'; // Max
                if q1_pos < 52 { plot[q1_pos] = '['; }
                if median_pos < 52 { plot[median_pos] = '|'; }
                if q3_pos < 52 { plot[q3_pos] = ']'; }
                
                let plot_str: String = plot.iter().collect();
                println!("{}", plot_str);
                println!("Min{}Q1{}Median{}Q3{}Max", 
                        " ".repeat(q1_pos.saturating_sub(3)),
                        " ".repeat(median_pos.saturating_sub(q1_pos).saturating_sub(6)),
                        " ".repeat(q3_pos.saturating_sub(median_pos).saturating_sub(6)),
                        " ".repeat(51_usize.saturating_sub(q3_pos).saturating_sub(3)));
            }
        }
        
        // Show distribution data summary if available but no quartiles calculated
        if let Some(ref csv_file_path) = analysis.results.distribution_data_file {
            if analysis.results.quartile_data.is_none() {
                println!("\n{}", "Distribution Data");
                println!("{}", "-".repeat(30));
                println!("Simulation data saved to: {}", csv_file_path);
                
                // Note: For detailed statistics, load the CSV file or view in external tools
                println!("Note: Load CSV file for detailed distribution statistics");
            }
        }
    }
    
    fn calculate_sensitivity_analysis(&self, analysis: &StackupAnalysis) -> Result<crate::sensitivity::SensitivityAnalysis> {
        let features = self.repository.get_features();
        
        // Convert feature contributions to stackup contributions for sensitivity analysis
        let stackup_contributions: Vec<StackupContribution> = analysis.feature_contributions.iter()
            .filter_map(|fc| {
                features.iter().find(|f| f.id == fc.feature_id).map(|feature| {
                    let component = self.repository.get_components()
                        .iter()
                        .find(|c| c.id == feature.component_id)
                        .map(|c| c.id.to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    
                    StackupContribution {
                        component_id: component,
                        feature_id: fc.feature_id,
                        direction: fc.direction,
                        half_count: fc.half_count,
                    }
                })
            })
            .collect();
        
        // Find the stackup
        let stackups = self.repository.get_stackups();
        let stackup = stackups.iter().find(|s| s.id == analysis.stackup_id)
            .ok_or_else(|| tessera_core::DesignTrackError::Validation("Stackup not found".to_string()))?;
        
        let sensitivity_analyzer = SensitivityAnalyzer::new(None);
        sensitivity_analyzer.analyze_stackup(stackup, &features, &stackup_contributions)
    }
    
    /// Calculate tolerance-normalized sensitivity (ignoring geometric multipliers)
    fn calculate_tolerance_normalized_sensitivity(&self, analysis: &StackupAnalysis) -> Result<Vec<(String, f64, f64)>> {
        let features = self.repository.get_features();
        
        let mut tolerance_contributions = Vec::new();
        let mut total_tolerance_variance = 0.0;
        
        // Calculate variance based purely on tolerance magnitude (ignoring direction multipliers)
        for fc in &analysis.feature_contributions {
            if let Some(feature) = features.iter().find(|f| f.id == fc.feature_id) {
                // Calculate tolerance variance ignoring geometric multipliers
                let total_tolerance = feature.tolerance.plus + feature.tolerance.minus;
                let tolerance_variance = match feature.tolerance.distribution {
                    ToleranceDistribution::Normal => (total_tolerance / 6.0).powi(2),
                    ToleranceDistribution::Uniform => (total_tolerance).powi(2) / 12.0,
                    ToleranceDistribution::Triangular => (total_tolerance).powi(2) / 24.0,
                    _ => (total_tolerance / 6.0).powi(2), // Default to normal
                };
                
                tolerance_contributions.push((feature.name.clone(), tolerance_variance, total_tolerance));
                total_tolerance_variance += tolerance_variance;
            }
        }
        
        // Calculate percentages and sort by contribution
        let mut result: Vec<(String, f64, f64)> = tolerance_contributions.into_iter()
            .map(|(name, variance, tolerance)| {
                let percentage = if total_tolerance_variance > 0.0 {
                    (variance / total_tolerance_variance) * 100.0
                } else {
                    0.0
                };
                (name, percentage, tolerance)
            })
            .collect();
        
        // Sort by percentage (descending)
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(result)
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
        let components = self.repository.get_components();
        let feature_contributions: Vec<FeatureContribution> = stackup.dimension_chain.iter()
            .filter_map(|&id| features.iter().find(|f| f.id == id))
            .map(|f| {
                let component = components.iter().find(|c| c.id == f.component_id);
                let feature_info = if let Some(comp) = component {
                    FeatureInfo::from_feature_and_component(f, comp)
                } else {
                    FeatureInfo {
                        feature_name: f.name.clone(),
                        feature_description: f.description.clone(),
                        component_name: "Unknown".to_string(),
                        component_description: "Unknown".to_string(),
                        feature_category: f.feature_category,
                        nominal: f.nominal,
                        tolerance_plus: f.tolerance.plus,
                        tolerance_minus: f.tolerance.minus,
                    }
                };
                
                FeatureContribution {
                    feature_id: f.id,
                    feature_name: f.name.clone(),
                    direction: 1.0,
                    half_count: false,
                    contribution_type: ContributionType::Additive,
                    feature_info,
                }
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
        let components = self.repository.get_components();
        
        if features.len() < 2 {
            println!("Need at least 2 features to create a mate.");
            return Ok(());
        }
        
        if components.is_empty() {
            println!("No components found. Add components first.");
            return Ok(());
        }
        
        let name = Text::new("Mate name:")
            .prompt()?;
        
        let description = Text::new("Description:")
            .prompt()?;
        
        // Select primary feature by component first
        println!("\nSelecting primary feature:");
        let component_options: Vec<String> = components.iter()
            .map(|c| format!("{} - {}", c.name, c.description))
            .collect();
        
        let primary_component_selection = Select::new("Select component for primary feature:", component_options.clone()).prompt()?;
        let primary_component_index = component_options.iter().position(|x| x == &primary_component_selection).unwrap();
        let primary_component = &components[primary_component_index];
        
        let primary_component_features = self.repository.get_features_for_component(primary_component.id);
        if primary_component_features.is_empty() {
            println!("No features found for component '{}'. Add features first.", primary_component.name);
            return Ok(());
        }
        
        let primary_feature_options: Vec<String> = primary_component_features.iter()
            .map(|f| format!("{} - {} ({}) [{}]", f.name, f.description, f.nominal, f.feature_category))
            .collect();
        
        let primary_feature_selection = Select::new("Select primary feature:", primary_feature_options.clone()).prompt()?;
        let primary_feature_index = primary_feature_options.iter().position(|x| x == &primary_feature_selection).unwrap();
        let primary_feature = &primary_component_features[primary_feature_index];
        
        // Select secondary feature by component first
        println!("\nSelecting secondary feature:");
        let secondary_component_selection = Select::new("Select component for secondary feature:", component_options.clone()).prompt()?;
        let secondary_component_index = component_options.iter().position(|x| x == &secondary_component_selection).unwrap();
        let secondary_component = &components[secondary_component_index];
        
        let secondary_component_features = self.repository.get_features_for_component(secondary_component.id);
        if secondary_component_features.is_empty() {
            println!("No features found for component '{}'. Add features first.", secondary_component.name);
            return Ok(());
        }
        
        let secondary_feature_options: Vec<String> = secondary_component_features.iter()
            .filter(|f| f.id != primary_feature.id) // Exclude the already selected primary feature
            .map(|f| format!("{} - {} ({}) [{}]", f.name, f.description, f.nominal, f.feature_category))
            .collect();
        
        if secondary_feature_options.is_empty() {
            println!("No available secondary features (different from primary feature).");
            return Ok(());
        }
        
        let secondary_feature_selection = Select::new("Select secondary feature:", secondary_feature_options.clone()).prompt()?;
        let secondary_feature_index = secondary_feature_options.iter().position(|x| x == &secondary_feature_selection).unwrap();
        
        // Find the actual feature from the filtered list
        let available_secondary_features: Vec<_> = secondary_component_features.iter()
            .filter(|f| f.id != primary_feature.id)
            .collect();
        let secondary_feature = available_secondary_features[secondary_feature_index];
        
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
        
        // Update mate with descriptive information and fit results
        mate.update_descriptive_info(primary_feature, primary_component, secondary_feature, secondary_component);
        
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
    
    pub async fn list_mates_interactive(&self) -> Result<()> {
        let mates = self.repository.get_mates();
        let features = self.repository.get_features();
        let components = self.repository.get_components();
        
        if mates.is_empty() {
            println!("No mates found");
            return Ok(());
        }
        
        if components.is_empty() {
            println!("No components found");
            return Ok(());
        }
        
        // Ask user to select a component to filter mates
        let mut component_options: Vec<String> = components.iter()
            .map(|c| format!("{} - {}", c.name, c.description))
            .collect();
        component_options.insert(0, "All components".to_string());
        
        let component_selection = Select::new("Select component (or all):", component_options.clone()).prompt()?;
        
        let filtered_mates: Vec<&crate::data::Mate> = if component_selection == "All components" {
            mates.iter().collect()
        } else {
            let component_index = component_options.iter().position(|x| x == &component_selection).unwrap() - 1; // -1 because we added "All" at index 0
            let selected_component = &components[component_index];
            
            // Filter mates that involve features from the selected component
            mates.iter().filter(|mate| {
                let primary_feature = features.iter().find(|f| f.id == mate.primary_feature);
                let secondary_feature = features.iter().find(|f| f.id == mate.secondary_feature);
                
                match (primary_feature, secondary_feature) {
                    (Some(pf), Some(sf)) => {
                        pf.component_id == selected_component.id || sf.component_id == selected_component.id
                    },
                    _ => false,
                }
            }).collect()
        };
        
        if filtered_mates.is_empty() {
            println!("No mates found for the selected filter");
            return Ok(());
        }
        
        println!("Mates:");
        for (i, mate) in filtered_mates.iter().enumerate() {
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
        let components = self.repository.get_components();
        if components.is_empty() {
            println!("No components found. Add components first.");
            return Ok(());
        }
        
        // First, select the component to narrow down features
        let component_options: Vec<String> = components.iter()
            .map(|c| format!("{} - {}", c.name, c.description))
            .collect();
        
        let component_selection = Select::new("Select component:", component_options.clone()).prompt()?;
        let component_index = component_options.iter().position(|x| x == &component_selection).unwrap();
        let selected_component = &components[component_index];
        
        // Get features for the selected component
        let component_features = self.repository.get_features_for_component(selected_component.id);
        if component_features.is_empty() {
            println!("No features found for component '{}'. Add features first.", selected_component.name);
            return Ok(());
        }
        
        let feature_options: Vec<String> = component_features.iter()
            .map(|f| format!("{} - {} ({})", f.name, f.description, f.nominal))
            .collect();
        
        let feature_selection = Select::new("Select feature to edit:", feature_options.clone()).prompt()?;
        let feature_index = feature_options.iter().position(|x| x == &feature_selection).unwrap();
        let mut feature = component_features[feature_index].clone();
        
        println!("Editing feature: {}", feature.name);
        
        let edit_options = vec![
            "Name",
            "Description",
            "Feature Category",
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
                "Feature Category" => {
                    let category_options = vec![
                        "External",
                        "Internal",
                    ];
                    
                    let current_category = format!("{}", feature.feature_category);
                    let category_str = Select::new("Feature category:", category_options)
                        .with_help_message(&format!("Current: {} (External=shafts/pins, Internal=holes/slots)", current_category))
                        .prompt()?;
                    
                    let new_category = match category_str {
                        "External" => FeatureCategory::External,
                        "Internal" => FeatureCategory::Internal,
                        _ => feature.feature_category,
                    };
                    
                    if new_category != feature.feature_category {
                        feature.feature_category = new_category;
                        feature.updated = chrono::Utc::now();
                        println!("✓ Feature category updated to: {}", new_category);
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
        
        // Update descriptive information before saving
        let components = self.repository.get_components();
        if let Some(primary_feature) = features.iter().find(|f| f.id == mate.primary_feature) {
            if let Some(secondary_feature) = features.iter().find(|f| f.id == mate.secondary_feature) {
                if let Some(primary_component) = components.iter().find(|c| c.id == primary_feature.component_id) {
                    if let Some(secondary_component) = components.iter().find(|c| c.id == secondary_feature.component_id) {
                        mate.update_descriptive_info(primary_feature, primary_component, secondary_feature, secondary_component);
                    }
                }
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
            "Upper/Lower Specification Limits (USL/LSL)",
            "Target Dimension (auto-calculated from USL/LSL)",
            "Legacy Tolerance Target (rarely needed)",
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
                "Upper/Lower Specification Limits (USL/LSL)" => {
                    println!("\nConfiguring engineering specification limits for process capability analysis:");
                    
                    let current_usl = stackup.upper_spec_limit.map(|v| v.to_string()).unwrap_or("None".to_string());
                    let current_lsl = stackup.lower_spec_limit.map(|v| v.to_string()).unwrap_or("None".to_string());
                    
                    println!("Current USL: {}", current_usl);
                    println!("Current LSL: {}", current_lsl);
                    
                    let use_spec_limits = Confirm::new("Set specification limits for process capability analysis?")
                        .with_default(stackup.has_specification_limits())
                        .with_help_message("Required for Cp/Cpk calculations")
                        .prompt()?;
                    
                    if use_spec_limits {
                        let usl_str = Text::new("Upper Specification Limit (USL):")
                            .with_default(&current_usl)
                            .with_help_message("Engineering upper limit for process capability")
                            .prompt()?;
                        
                        let lsl_str = Text::new("Lower Specification Limit (LSL):")
                            .with_default(&current_lsl)
                            .with_help_message("Engineering lower limit for process capability")
                            .prompt()?;
                        
                        let usl = if usl_str != "None" { 
                            usl_str.parse::<f64>().ok() 
                        } else { 
                            None 
                        };
                        
                        let lsl = if lsl_str != "None" { 
                            lsl_str.parse::<f64>().ok() 
                        } else { 
                            None 
                        };
                        
                        stackup.set_specification_limits(usl, lsl);
                        
                        if let (Some(usl_val), Some(lsl_val)) = (usl, lsl) {
                            println!("✓ Specification limits set: USL = {:.6}, LSL = {:.6}", usl_val, lsl_val);
                            println!("✓ Target dimension auto-calculated: {:.6}", stackup.target_dimension);
                        } else {
                            println!("⚠️  Incomplete specification limits - process capability analysis will not be available");
                        }
                    } else {
                        stackup.set_specification_limits(None, None);
                        println!("✓ Specification limits cleared");
                    }
                },
                "Target Dimension (auto-calculated from USL/LSL)" => {
                    let target_str = Text::new("New target dimension:")
                        .with_default(&stackup.target_dimension.to_string())
                        .prompt()?;
                    if let Ok(target) = target_str.parse::<f64>() {
                        stackup.target_dimension = target;
                        stackup.updated = chrono::Utc::now();
                    }
                },
                "Legacy Tolerance Target (rarely needed)" => {
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
    
    pub async fn delete_stackup_interactive(&mut self) -> Result<()> {
        let stackups = self.repository.get_stackups();
        if stackups.is_empty() {
            println!("No stackups found.");
            return Ok(());
        }
        
        let stackup_options: Vec<String> = stackups.iter()
            .map(|s| format!("{} - {} ({} features)", s.name, s.description, s.dimension_chain.len()))
            .collect();
        
        let stackup_selection = Select::new("Select stackup to delete:", stackup_options.clone()).prompt()?;
        let stackup_index = stackup_options.iter().position(|x| x == &stackup_selection).unwrap();
        let stackup_to_delete = stackups[stackup_index].clone(); // Clone to avoid borrow issues
        
        // Show stackup details and confirm deletion
        println!("\nStackup to delete:");
        println!("  Name: {}", stackup_to_delete.name);
        println!("  Description: {}", stackup_to_delete.description);
        println!("  Features: {}", stackup_to_delete.dimension_chain.len());
        
        // Count associated analyses
        let associated_analyses = self.repository.get_analyses_for_stackup(stackup_to_delete.id);
        if !associated_analyses.is_empty() {
            println!("  Associated analyses: {} (will also be deleted)", associated_analyses.len());
        }
        
        let confirm_delete = Confirm::new("⚠️  Are you sure you want to delete this stackup?")
            .with_default(false)
            .with_help_message("This action cannot be undone")
            .prompt()?;
        
        if confirm_delete {
            self.repository.delete_stackup(stackup_to_delete.id)?;
            
            let tol_dir = self.project_context.module_path("tol");
            self.repository.save_to_directory(&tol_dir)?;
            
            println!("✓ Stackup '{}' deleted successfully!", stackup_to_delete.name);
        } else {
            println!("Delete operation cancelled.");
        }
        
        Ok(())
    }
    
    async fn configure_feature_contribution(&mut self, stackup: &mut Stackup, feature: &Feature) -> Result<()> {
        println!("\nConfiguring vector contribution for feature: {} ({:.3})", feature.name, feature.nominal);
        
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
        
        stackup.update_feature_contribution(feature.id, direction, half_count, contribution_type);
        
        println!("✓ Vector contribution configured: {:.2} (half: {})", direction, half_count);
        
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
                    
                    stackup.add_dimension(selected_feature.id, selected_feature.name.clone());
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
                    
                    stackup.remove_dimension(selected_feature.id);
                    println!("Removed feature: {}", selected_feature.name);
                },
                "List Current Features" => {
                    if stackup.dimension_chain.is_empty() {
                        println!("No features in stackup.");
                    } else {
                        println!("Current features in stackup:");
                        for (i, &feature_id) in stackup.dimension_chain.iter().enumerate() {
                            if let Some(feature) = features.iter().find(|f| f.id == feature_id) {
                                let contribution = stackup.feature_contributions.iter().find(|c| c.feature_id == feature_id);
                                let contribution_info = if let Some(contrib) = contribution {
                                    let half_info = if contrib.half_count { " (half)" } else { "" };
                                    format!(" [direction: {:.2}{}]", contrib.direction, half_info)
                                } else {
                                    " [no contribution data]".to_string()
                                };
                                println!("  {}. {} ({:.3}){}", i + 1, feature.name, feature.nominal, contribution_info);
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
    
    pub async fn list_features_interactive(&self) -> Result<()> {
        let components = self.repository.get_components();
        if components.is_empty() {
            println!("No components found. Add components first.");
            return Ok(());
        }
        
        let features = self.repository.get_features();
        if features.is_empty() {
            println!("No features found. Add features first.");
            return Ok(());
        }
        
        // Ask user to select viewing option
        let view_options = vec![
            "List all features",
            "List features for a specific component",
        ];
        
        let view_selection = Select::new("How would you like to view features?", view_options).prompt()?;
        
        match view_selection {
            "List all features" => {
                self.list_all_features()?;
            },
            "List features for a specific component" => {
                let component_options: Vec<String> = components.iter()
                    .map(|c| format!("{} - {}", c.name, c.description))
                    .collect();
                
                let component_selection = Select::new("Select component:", component_options.clone()).prompt()?;
                let component_index = component_options.iter().position(|x| x == &component_selection).unwrap();
                let selected_component = &components[component_index];
                
                let component_features = self.repository.get_features_for_component(selected_component.id);
                if component_features.is_empty() {
                    println!("No features found for component '{}'", selected_component.name);
                } else {
                    println!("\nFeatures for component '{}':", selected_component.name);
                    for (i, feature) in component_features.iter().enumerate() {
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
                        
                        println!("{}. {} - {} ({:.3} +{:.3}/-{:.3}){}{}",
                                 i + 1, feature.name, feature.description, 
                                 feature.nominal, feature.tolerance.plus, feature.tolerance.minus,
                                 distribution_info, drawing_location);
                        println!("   Type: {:?}, Category: {}", feature.feature_type, feature.feature_category);
                        println!("   ID: {}", feature.id);
                        println!();
                    }
                }
            },
            _ => {}
        }
        
        Ok(())
    }
    
    fn list_all_features(&self) -> Result<()> {
        let features = self.repository.get_features();
        let components = self.repository.get_components();
        
        println!("\nAll Features:");
        for (i, feature) in features.iter().enumerate() {
            let component = components.iter().find(|c| c.id == feature.component_id);
            let component_name = component.map(|c| c.name.as_str()).unwrap_or("Unknown");
            
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
            
            println!("{}. {} - {} ({:.3} +{:.3}/-{:.3}){}{}",
                     i + 1, feature.name, feature.description, 
                     feature.nominal, feature.tolerance.plus, feature.tolerance.minus,
                     distribution_info, drawing_location);
            println!("   Component: {}", component_name);
            println!("   Type: {:?}, Category: {}", feature.feature_type, feature.feature_category);
            println!("   ID: {}", feature.id);
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

    pub async fn view_analysis_interactive(&mut self) -> Result<()> {
        let analyses = self.repository.get_analyses();
        if analyses.is_empty() {
            println!("No analysis results found. Run stackup analyses first.");
            return Ok(());
        }
        
        // Create display options for each analysis
        let analysis_options: Vec<String> = analyses.iter()
            .map(|a| format!("{} - {} ({:?}) [{}]", 
                a.stackup_name, 
                a.created.format("%Y-%m-%d %H:%M:%S"),
                a.config.method,
                if a.results.cp >= 1.33 { "Good" } else if a.results.cp >= 1.0 { "Fair" } else { "Poor" }
            ))
            .collect();
        
        // Allow user to select an analysis to view
        let analysis_selection = Select::new("Select analysis to view:", analysis_options.clone()).prompt()?;
        let analysis_index = analysis_options.iter().position(|x| x == &analysis_selection).unwrap();
        let selected_analysis = &analyses[analysis_index];
        
        // Display the selected analysis results
        self.display_analysis_results(selected_analysis);
        
        Ok(())
    }
    
    pub async fn delete_analysis_interactive(&mut self) -> Result<()> {
        let analyses = self.repository.get_analyses();
        if analyses.is_empty() {
            println!("No analysis results found.");
            return Ok(());
        }
        
        // Create display options for each analysis
        let analysis_options: Vec<String> = analyses.iter()
            .map(|a| format!("{} - {} ({:?}) [{}]", 
                a.stackup_name, 
                a.created.format("%Y-%m-%d %H:%M:%S"),
                a.config.method,
                if a.results.cp >= 1.33 { "Good" } else if a.results.cp >= 1.0 { "Fair" } else { "Poor" }
            ))
            .collect();
        
        // Allow user to select an analysis to delete
        let analysis_selection = Select::new("Select analysis to delete:", analysis_options.clone()).prompt()?;
        let analysis_index = analysis_options.iter().position(|x| x == &analysis_selection).unwrap();
        let selected_analysis = &analyses[analysis_index];
        
        // Show analysis details and confirm deletion
        println!("\nAnalysis to delete:");
        println!("  Stackup: {}", selected_analysis.stackup_name);
        println!("  Method: {:?}", selected_analysis.config.method);
        println!("  Created: {}", selected_analysis.created.format("%Y-%m-%d %H:%M:%S"));
        println!("  Cp: {:.3}", selected_analysis.results.cp);
        
        let confirm_delete = Confirm::new("⚠️  Are you sure you want to delete this analysis?")
            .with_default(false)
            .with_help_message("This action cannot be undone")
            .prompt()?;
        
        if confirm_delete {
            self.repository.delete_analysis(selected_analysis.created)?;
            
            let tol_dir = self.project_context.module_path("tol");
            self.repository.save_to_directory(&tol_dir)?;
            
            println!("✓ Analysis deleted successfully!");
        } else {
            println!("Delete operation cancelled.");
        }
        
        Ok(())
    }
}