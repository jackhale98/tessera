use anyhow::Result;
use plotters::prelude::*;
// SVGBackend is included with svg_backend feature
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::data::{StackupAnalysis, Feature, Stackup, Component};

/// Represents different export formats for plots
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlotFormat {
    Svg,
    Html,
}

impl PlotFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            PlotFormat::Svg => "svg",
            PlotFormat::Html => "html",
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            PlotFormat::Svg => "image/svg+xml",
            PlotFormat::Html => "text/html",
        }
    }
}

/// Configuration for plot generation
#[derive(Debug, Clone)]
pub struct PlotConfig {
    pub width: u32,
    pub height: u32,
    pub title_font_size: u32,
    pub axis_font_size: u32,
    pub label_font_size: u32,
}

impl Default for PlotConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            title_font_size: 24,
            axis_font_size: 16,
            label_font_size: 14,
        }
    }
}

/// Main plotting interface for tolerance analysis
pub struct TolerancePlotter {
    config: PlotConfig,
}

impl TolerancePlotter {
    pub fn new() -> Self {
        Self {
            config: PlotConfig::default(),
        }
    }

    pub fn with_config(config: PlotConfig) -> Self {
        Self { config }
    }

    /// Read Monte Carlo simulation data from CSV file
    fn read_monte_carlo_data(&self, csv_file_path: &str) -> Result<Vec<f64>> {
        let content = fs::read_to_string(csv_file_path)
            .map_err(|e| anyhow::anyhow!("Failed to read CSV file {}: {}", csv_file_path, e))?;
        
        let mut data = Vec::new();
        for line in content.lines().skip(1) { // Skip header
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 2 {
                // Parse the dimension_value (second column)
                if let Ok(value) = parts[1].parse::<f64>() {
                    data.push(value);
                }
            }
        }
        
        if data.is_empty() {
            return Err(anyhow::anyhow!("No valid data found in CSV file"));
        }
        
        Ok(data)
    }

    /// Export histogram plot for Monte Carlo analysis results
    pub fn export_histogram(
        &self,
        analysis: &StackupAnalysis,
        stackup: &Stackup,
        format: PlotFormat,
        output_path: &Path,
    ) -> Result<()> {
        // Check if we have Monte Carlo data by checking if there's a CSV file
        let _csv_file = analysis.results.distribution_data_file.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No Monte Carlo results available for histogram"))?;

        match format {
            PlotFormat::Svg => self.create_histogram_svg(analysis, stackup, output_path),
            PlotFormat::Html => self.create_histogram_html(analysis, stackup, output_path),
        }
    }

    /// Export waterfall plot showing feature contributions
    pub fn export_waterfall(
        &self,
        analysis: &StackupAnalysis,
        stackup: &Stackup,
        features: &HashMap<uuid::Uuid, Feature>,
        components: &HashMap<uuid::Uuid, Component>,
        format: PlotFormat,
        output_path: &Path,
    ) -> Result<()> {
        match format {
            PlotFormat::Svg => self.create_waterfall_svg(analysis, stackup, features, components, output_path),
            PlotFormat::Html => self.create_waterfall_html(analysis, stackup, features, components, output_path),
        }
    }

    /// Create SVG histogram
    fn create_histogram_svg(
        &self,
        analysis: &StackupAnalysis,
        stackup: &Stackup,
        output_path: &Path,
    ) -> Result<()> {
        // Read Monte Carlo data from CSV file
        let csv_file = analysis.results.distribution_data_file.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No Monte Carlo data file available"))?;
        let simulation_data = self.read_monte_carlo_data(csv_file)?;
        
        // Create output directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let backend = SVGBackend::new(output_path, (self.config.width, self.config.height));
        let root = backend.into_drawing_area();
        root.fill(&WHITE)?;

        let title = format!("Monte Carlo Analysis: {}", stackup.name);
        
        // Create proper histogram bins
        let bin_count = 50; // Good resolution for most datasets
        let min_val = simulation_data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_val = simulation_data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let bin_width = (max_val - min_val) / bin_count as f64;
        
        // Calculate histogram
        let mut histogram = vec![0; bin_count];
        for &value in &simulation_data {
            let bin_index = ((value - min_val) / bin_width).floor() as usize;
            let bin_index = bin_index.min(bin_count - 1);
            histogram[bin_index] += 1;
        }
        
        let max_count = *histogram.iter().max().unwrap_or(&1);

        let mut chart = ChartBuilder::on(&root)
            .caption(&title, ("Arial", self.config.title_font_size))
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(min_val..max_val, 0..max_count)?;

        chart
            .configure_mesh()
            .x_desc("Stackup Value")
            .y_desc("Frequency")
            .axis_desc_style(("Arial", self.config.axis_font_size))
            .y_max_light_lines(5)  // Lighter grid lines
            .x_max_light_lines(5)
            .draw()?;

        // Draw histogram bars
        chart.draw_series(
            histogram
                .iter()
                .enumerate()
                .map(|(i, &count)| {
                    let x = min_val + (i as f64 + 0.5) * bin_width;
                    Rectangle::new([(x - bin_width/2.0, 0), (x + bin_width/2.0, count)], BLUE.filled())
                })
        )?;

        // Draw upper and lower specification limit lines if specified
        if let Some(usl) = stackup.upper_spec_limit {
            chart.draw_series(std::iter::once(PathElement::new(
                vec![(usl, 0), (usl, max_count)],
                RED.stroke_width(2),
            )))?
            .label("Upper Spec Limit")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], RED.stroke_width(2)));
        }
        
        if let Some(lsl) = stackup.lower_spec_limit {
            chart.draw_series(std::iter::once(PathElement::new(
                vec![(lsl, 0), (lsl, max_count)],
                RED.stroke_width(2),
            )))?
            .label("Lower Spec Limit")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], RED.stroke_width(2)));
        }

        // Configure legend if any specification limits are present
        if stackup.upper_spec_limit.is_some() || stackup.lower_spec_limit.is_some() {
            chart.configure_series_labels().draw()?;
        }

        // Add statistics annotations
        let mean = simulation_data.iter().sum::<f64>() / simulation_data.len() as f64;
        let variance = simulation_data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / simulation_data.len() as f64;
        let std_dev = variance.sqrt();
        
        let stats_text = format!(
            "Mean: {:.4}\nStd Dev: {:.4}\nCp: {:.3}\nCpk: {:.3}\nSamples: {}",
            mean,
            std_dev,
            analysis.results.cp,
            analysis.results.cpk,
            simulation_data.len()
        );

        root.titled(&stats_text, ("Arial", self.config.label_font_size))?;
        root.present()?;

        Ok(())
    }

    /// Create SVG histogram as string for embedding
    fn create_histogram_svg_string(&self, analysis: &StackupAnalysis, stackup: &Stackup) -> Result<String> {
        // Read Monte Carlo data from CSV file
        let csv_file = analysis.results.distribution_data_file.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No Monte Carlo data file available"))?;
        let simulation_data = self.read_monte_carlo_data(csv_file)?;
        
        let mut svg_string = String::new();
        {
            let backend = SVGBackend::with_string(&mut svg_string, (self.config.width, self.config.height));
            let root = backend.into_drawing_area();
            root.fill(&WHITE)?;

            let title = format!("Monte Carlo Analysis: {}", stackup.name);
            
            // Create proper histogram bins
            let bin_count = 30; // Slightly fewer bins for SVG string to keep size manageable
            let min_val = simulation_data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_val = simulation_data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let bin_width = (max_val - min_val) / bin_count as f64;
            
            // Calculate histogram
            let mut histogram = vec![0; bin_count];
            for &value in &simulation_data {
                let bin_index = ((value - min_val) / bin_width).floor() as usize;
                let bin_index = bin_index.min(bin_count - 1);
                histogram[bin_index] += 1;
            }
            
            let max_count = *histogram.iter().max().unwrap_or(&1);

            let mut chart = ChartBuilder::on(&root)
                .caption(&title, ("Arial", self.config.title_font_size))
                .margin(10)
                .x_label_area_size(40)
                .y_label_area_size(50)
                .build_cartesian_2d(min_val..max_val, 0..max_count)?;

            chart
                .configure_mesh()
                .x_desc("Stackup Value")
                .y_desc("Frequency")
                .axis_desc_style(("Arial", self.config.axis_font_size))
                .draw()?;

            // Draw histogram bars
            chart.draw_series(
                histogram
                    .iter()
                    .enumerate()
                    .map(|(i, &count)| {
                        let x = min_val + (i as f64 + 0.5) * bin_width;
                        Rectangle::new([(x - bin_width/2.0, 0), (x + bin_width/2.0, count)], BLUE.filled())
                    })
            )?;

            // Draw upper and lower specification limit lines if specified
            if let Some(usl) = stackup.upper_spec_limit {
                chart.draw_series(std::iter::once(PathElement::new(
                    vec![(usl, 0), (usl, max_count)],
                    RED.stroke_width(2),
                )))?;
            }
            
            if let Some(lsl) = stackup.lower_spec_limit {
                chart.draw_series(std::iter::once(PathElement::new(
                    vec![(lsl, 0), (lsl, max_count)],
                    RED.stroke_width(2),
                )))?;
            }

            root.present()?;
        }
        Ok(svg_string)
    }

    /// Create SVG waterfall as string for embedding
    fn create_waterfall_svg_string(&self, analysis: &StackupAnalysis, stackup: &Stackup, features: &HashMap<uuid::Uuid, Feature>, components: &HashMap<uuid::Uuid, Component>) -> Result<String> {
        let mut svg_string = String::new();
        {
            let backend = SVGBackend::with_string(&mut svg_string, (self.config.width, self.config.height));
            let root = backend.into_drawing_area();
            root.fill(&WHITE)?;

            let title = format!("Waterfall Analysis: {}", stackup.name);

            // Prepare waterfall data with component information
            let mut contributions = Vec::new();
            let mut cumulative: f64 = 0.0;
            
            for feature_contrib in &analysis.feature_contributions {
                if let Some(feature) = features.get(&feature_contrib.feature_id.into()) {
                    let component_name = components.get(&feature.component_id.into())
                        .map(|c| c.name.clone())
                        .unwrap_or_else(|| "Unknown".to_string());
                    
                    let nominal = feature.nominal * feature_contrib.direction;
                    cumulative += nominal;
                    
                    // Store as (component_name, feature_name, contribution, cumulative)
                    contributions.push((component_name, feature.name.clone(), nominal, cumulative));
                }
            }

            let max_value: f64 = cumulative.abs().max(contributions.iter().map(|(_, _, _, cum)| cum.abs()).fold(0.0_f64, f64::max));
            let margin = max_value * 0.1;

            let mut chart = ChartBuilder::on(&root)
                .caption(&title, ("Arial", self.config.title_font_size))
                .margin(10)
                .x_label_area_size(100)  // Increased for feature names
                .y_label_area_size(60)
                .build_cartesian_2d(0f64..(contributions.len() + 1) as f64, (-max_value - margin)..(max_value + margin))?;

            chart
                .configure_mesh()
                .x_desc("Features")
                .y_desc("Value")
                .axis_desc_style(("Arial", self.config.axis_font_size))
                .draw()?;

            // Draw waterfall bars with component and feature labels
            let mut prev_cumulative = 0.0;
            for (i, (_component_name, _feature_name, contribution, cumulative)) in contributions.iter().enumerate() {
                let x = (i + 1) as f64;  // Start at x=1 to leave space for labels
                let color = if *contribution >= 0.0 { &GREEN } else { &RED };
                
                // Draw the bar
                chart.draw_series(std::iter::once(Rectangle::new(
                    [(x - 0.3, prev_cumulative), (x + 0.3, *cumulative)],
                    color.filled(),
                )))?;

                prev_cumulative = *cumulative;
            }

            // Draw cumulative line plot
            if contributions.len() > 1 {
                let line_points: Vec<(f64, f64)> = std::iter::once((0.0, 0.0)) // Start at origin
                    .chain(contributions.iter().enumerate().map(|(i, (_, _, _, cumulative))| {
                        ((i + 1) as f64, *cumulative)
                    }))
                    .collect();

                chart.draw_series(std::iter::once(PathElement::new(line_points, RED.stroke_width(2))))?;
            }

            root.present()?;
        }
        Ok(svg_string)
    }

    /// Create HTML histogram with interactive features
    fn create_histogram_html(
        &self,
        analysis: &StackupAnalysis,
        stackup: &Stackup,
        output_path: &Path,
    ) -> Result<()> {
        // Read Monte Carlo data from CSV file
        let csv_file = analysis.results.distribution_data_file.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No Monte Carlo data file available"))?;
        let simulation_data = self.read_monte_carlo_data(csv_file)?;
        
        // Create output directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let title = format!("Monte Carlo Analysis: {}", stackup.name);
        
        // Generate SVG plot as string to embed in HTML
        let svg_content = self.create_histogram_svg_string(analysis, stackup)?;
        
        // Generate HTML with embedded SVG and interactivity
        let html_content = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>{title}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .container {{ max-width: 1000px; margin: 0 auto; }}
        .stats {{ 
            background: #f5f5f5; 
            padding: 15px; 
            margin: 20px 0; 
            border-radius: 5px; 
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 10px;
        }}
        .stat-item {{ text-align: center; }}
        .stat-value {{ font-size: 1.5em; font-weight: bold; color: #2196F3; }}
        .stat-label {{ color: #666; }}
        svg {{ border: 1px solid #ddd; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>{title}</h1>
        
        <div class="stats">
            <div class="stat-item">
                <div class="stat-value">{mean:.4}</div>
                <div class="stat-label">Mean</div>
            </div>
            <div class="stat-item">
                <div class="stat-value">{std_dev:.4}</div>
                <div class="stat-label">Std Deviation</div>
            </div>
            <div class="stat-item">
                <div class="stat-value">{cp:.3}</div>
                <div class="stat-label">Cp</div>
            </div>
            <div class="stat-item">
                <div class="stat-value">{cpk:.3}</div>
                <div class="stat-label">Cpk</div>
            </div>
            <div class="stat-item">
                <div class="stat-value">{sample_count}</div>
                <div class="stat-label">Samples</div>
            </div>
            <div class="stat-item">
                <div class="stat-value">{confidence:.0}%</div>
                <div class="stat-label">Confidence</div>
            </div>
        </div>
        
        <div id="histogram">
            {svg_content}
        </div>
        
        <div style="margin-top: 20px;">
            <h3>Analysis Details</h3>
            <p><strong>Stackup:</strong> {stackup_name}</p>
            <p><strong>Analysis Method:</strong> Monte Carlo Simulation</p>
            <p><strong>Distribution Type:</strong> Various (feature-dependent)</p>
        </div>
    </div>
</body>
</html>"#,
            title = title,
            mean = simulation_data.iter().sum::<f64>() / simulation_data.len() as f64,
            std_dev = {
                let mean = simulation_data.iter().sum::<f64>() / simulation_data.len() as f64;
                let variance = simulation_data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / simulation_data.len() as f64;
                variance.sqrt()
            },
            cp = analysis.results.cp,
            cpk = analysis.results.cpk,
            sample_count = simulation_data.len(),
            confidence = "95",
            stackup_name = stackup.name,
            svg_content = svg_content,
        );

        fs::write(output_path, html_content)?;
        Ok(())
    }

    /// Create SVG waterfall plot
    fn create_waterfall_svg(
        &self,
        analysis: &StackupAnalysis,
        stackup: &Stackup,
        features: &HashMap<uuid::Uuid, Feature>,
        components: &HashMap<uuid::Uuid, Component>,
        output_path: &Path,
    ) -> Result<()> {
        // Create output directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let backend = SVGBackend::new(output_path, (self.config.width, self.config.height));
        let root = backend.into_drawing_area();
        root.fill(&WHITE)?;

        let title = format!("Waterfall Analysis: {}", stackup.name);

        // Prepare waterfall data with component information
        let mut contributions = Vec::new();
        let mut cumulative: f64 = 0.0;
        
        for feature_contrib in &analysis.feature_contributions {
            if let Some(feature) = features.get(&feature_contrib.feature_id.into()) {
                let component_name = components.get(&feature.component_id.into())
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                
                let nominal = feature.nominal * feature_contrib.direction;
                cumulative += nominal;
                
                // Store as (component_name, feature_name, contribution, cumulative)
                contributions.push((component_name, feature.name.clone(), nominal, cumulative));
            }
        }

        let max_value: f64 = cumulative.abs().max(contributions.iter().map(|(_, _, _, cum)| cum.abs()).fold(0.0_f64, f64::max));
        let margin = max_value * 0.1;

        let mut chart = ChartBuilder::on(&root)
            .caption(&title, ("Arial", self.config.title_font_size))
            .margin(10)
            .x_label_area_size(100)  // Increased for feature names
            .y_label_area_size(60)
            .build_cartesian_2d(0f64..(contributions.len() + 1) as f64, (-max_value - margin)..(max_value + margin))?;

        chart
            .configure_mesh()
            .x_desc("Features")
            .y_desc("Value")
            .axis_desc_style(("Arial", self.config.axis_font_size))
            .x_label_formatter(&|x| {
                let index = (*x).round() as usize;
                if index > 0 && index <= contributions.len() {
                    // Get the component name from contributions
                    contributions.get(index - 1)
                        .map(|(comp_name, _, _, _)| {
                            // Truncate long names for better display
                            if comp_name.len() > 8 {
                                format!("{}...", &comp_name[..5])
                            } else {
                                comp_name.clone()
                            }
                        })
                        .unwrap_or_else(|| format!("C{}", index))
                } else {
                    String::new()
                }
            })
            .axis_desc_style(("Arial", self.config.axis_font_size))
            .draw()?;

        // Draw waterfall bars with component and feature labels
        let mut prev_cumulative = 0.0;
        for (i, (_component_name, feature_name, contribution, cumulative)) in contributions.iter().enumerate() {
            let x = (i + 1) as f64;  // Start at x=1 to leave space for labels
            let color = if *contribution >= 0.0 { &GREEN } else { &RED };
            
            // Draw the bar
            chart.draw_series(std::iter::once(Rectangle::new(
                [(x - 0.3, prev_cumulative), (x + 0.3, *cumulative)],
                color.filled(),
            )))?;

            // Add feature name below the bar (smaller font, rotated)
            chart.draw_series(std::iter::once(Text::new(
                feature_name.clone(),
                (x, -max_value * 0.15),
                ("Arial", self.config.label_font_size - 4).into_font()
                    .transform(FontTransform::Rotate90)
                    .color(&BLACK),
            )))?;

            // Add running cumulative value label on the bar
            let mid_y = (prev_cumulative + cumulative) / 2.0;
            chart.draw_series(std::iter::once(Text::new(
                format!("{:.3}", cumulative),
                (x, mid_y),
                ("Arial", self.config.label_font_size - 2).into_font().color(&WHITE),
            )))?;

            // Draw connecting line to next bar (if not the last)
            if i < contributions.len() - 1 {
                chart.draw_series(std::iter::once(PathElement::new(
                    vec![(x + 0.3, *cumulative), (x + 0.7, *cumulative)],
                    BLACK.stroke_width(1),
                )))?;
            }

            prev_cumulative = *cumulative;
        }

        // Draw cumulative line plot
        if contributions.len() > 1 {
            let line_points: Vec<(f64, f64)> = std::iter::once((0.0, 0.0)) // Start at origin
                .chain(contributions.iter().enumerate().map(|(i, (_, _, _, cumulative))| {
                    ((i + 1) as f64, *cumulative)
                }))
                .collect();

            chart.draw_series(std::iter::once(PathElement::new(line_points, RED.stroke_width(2))))?;
        }

        root.present()?;
        Ok(())
    }

    /// Create HTML waterfall plot
    fn create_waterfall_html(
        &self,
        analysis: &StackupAnalysis,
        stackup: &Stackup,
        features: &HashMap<uuid::Uuid, Feature>,
        components: &HashMap<uuid::Uuid, Component>,
        output_path: &Path,
    ) -> Result<()> {
        // Create output directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let title = format!("Waterfall Analysis: {}", stackup.name);
        
        // Generate SVG plot as string to embed in HTML
        let svg_content = self.create_waterfall_svg_string(analysis, stackup, features, components)?;
        
        // Prepare data for HTML table using the actual analysis cumulative values
        let mut table_rows = String::new();
        
        // Use the same calculation as the waterfall plot to ensure consistency
        let mut cumulative: f64 = 0.0;
        for feature_contrib in &analysis.feature_contributions {
            if let Some(feature) = features.get(&feature_contrib.feature_id.into()) {
                let component_name = components.get(&feature.component_id.into())
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                
                let contribution = feature.nominal * feature_contrib.direction;
                cumulative += contribution;
                
                table_rows.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td><td>{:.4}</td><td>{:.4}</td><td>{:.4}</td></tr>",
                    component_name, feature.name, contribution, feature.tolerance.plus + feature.tolerance.minus, cumulative
                ));
            }
        }
        
        // Use the actual analysis result for the final nominal value display
        let final_nominal_from_analysis = analysis.results.nominal_dimension;

        let html_content = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>{title}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .container {{ max-width: 1000px; margin: 0 auto; }}
        table {{ width: 100%; border-collapse: collapse; margin: 20px 0; }}
        th, td {{ border: 1px solid #ddd; padding: 12px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
        tr:nth-child(even) {{ background-color: #f9f9f9; }}
        .summary {{ background: #e3f2fd; padding: 15px; margin: 20px 0; border-radius: 5px; }}
        .positive {{ color: #4CAF50; }}
        .negative {{ color: #f44336; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>{title}</h1>
        
        <div class="summary">
            <h3>Stackup Summary</h3>
            <p><strong>Final Nominal Value:</strong> {final_nominal:.6}</p>
            <p><strong>Calculated Nominal:</strong> {calculated_nominal:.6}</p>
            <p><strong>Total Features:</strong> {feature_count}</p>
            <p><strong>Analysis Type:</strong> {analysis_type}</p>
        </div>
        
        <div style="text-align: center; margin: 20px 0;">
            <h3>Waterfall Chart</h3>
            {svg_content}
        </div>
        
        <h3>Feature Contributions</h3>
        <table>
            <thead>
                <tr>
                    <th>Component</th>
                    <th>Feature</th>
                    <th>Contribution</th>
                    <th>Tolerance (±)</th>
                    <th>Cumulative</th>
                </tr>
            </thead>
            <tbody>
                {table_rows}
            </tbody>
        </table>
        
        <div style="margin-top: 20px;">
            <h3>Analysis Method</h3>
            <p>{method_description}</p>
        </div>
    </div>
</body>
</html>"#,
            title = title,
            final_nominal = final_nominal_from_analysis,
            calculated_nominal = final_nominal_from_analysis,
            feature_count = analysis.feature_contributions.len(),
            analysis_type = match analysis.results.distribution_data_file {
                Some(_) => "Monte Carlo + Waterfall",
                None => "Waterfall Only"
            },
            table_rows = table_rows,
            svg_content = svg_content,
            method_description = "This waterfall chart shows how individual feature contributions accumulate to the final stackup value. Each bar represents the cumulative effect of adding the current feature to the stackup chain."
        );

        fs::write(output_path, html_content)?;
        Ok(())
    }
}

impl Default for TolerancePlotter {
    fn default() -> Self {
        Self::new()
    }
}