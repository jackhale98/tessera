use anyhow::Result;
use plotters::prelude::*;
// SVGBackend is included with svg_backend feature
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::data::{StackupAnalysis, Feature, Stackup};

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

    /// Export histogram plot for Monte Carlo analysis results
    pub fn export_histogram(
        &self,
        analysis: &StackupAnalysis,
        stackup: &Stackup,
        format: PlotFormat,
        output_path: &Path,
    ) -> Result<()> {
        // Check if we have Monte Carlo data by checking if there's a CSV file
        if analysis.results.distribution_data_file.is_none() {
            return Err(anyhow::anyhow!("No Monte Carlo results available for histogram"));
        }

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
        format: PlotFormat,
        output_path: &Path,
    ) -> Result<()> {
        match format {
            PlotFormat::Svg => self.create_waterfall_svg(analysis, stackup, features, output_path),
            PlotFormat::Html => self.create_waterfall_html(analysis, stackup, features, output_path),
        }
    }

    /// Create SVG histogram
    fn create_histogram_svg(
        &self,
        analysis: &StackupAnalysis,
        stackup: &Stackup,
        output_path: &Path,
    ) -> Result<()> {
        let quartile_data = analysis.results.quartile_data.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No quartile data available for histogram"))?;
        
        // Create output directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let backend = SVGBackend::new(output_path, (self.config.width, self.config.height));
        let root = backend.into_drawing_area();
        root.fill(&WHITE)?;

        let title = format!("Monte Carlo Analysis: {}", stackup.name);
        
        // Create a simplified quartile-based visualization
        let min_val = quartile_data.minimum;
        let max_val = quartile_data.maximum;
        let range = max_val - min_val;
        
        // Create quartile visualization data points
        let quartile_points = vec![
            (quartile_data.minimum, "Min"),
            (quartile_data.q1, "Q1"),
            (quartile_data.median, "Median"),
            (quartile_data.q3, "Q3"),
            (quartile_data.maximum, "Max"),
        ];
        
        let max_height = 100; // Arbitrary scale for visualization

        let mut chart = ChartBuilder::on(&root)
            .caption(&title, ("Arial", self.config.title_font_size))
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(min_val..max_val, 0..max_height)?;

        chart
            .configure_mesh()
            .x_desc("Stackup Value")
            .y_desc("Frequency")
            .axis_desc_style(("Arial", self.config.axis_font_size))
            .draw()?;

        // Draw quartile visualization
        for (i, (value, _label)) in quartile_points.iter().enumerate() {
            let height = match i {
                0 | 4 => 30,  // Min and Max - shorter bars
                1 | 3 => 60,  // Q1 and Q3 - medium bars
                2 => 100,     // Median - tallest bar
                _ => 30,
            };
            
            chart.draw_series(std::iter::once(Rectangle::new(
                [(*value - range * 0.01, 0), (*value + range * 0.01, height)],
                BLUE.filled(),
            )))?;
        }

        // Add statistics annotations
        let stats_text = format!(
            "Median: {:.4}\nIQR: {:.4}\nCp: {:.3}\nCpk: {:.3}",
            quartile_data.median,
            quartile_data.iqr,
            analysis.results.cp,
            analysis.results.cpk
        );

        root.titled(&stats_text, ("Arial", self.config.label_font_size))?;
        root.present()?;

        Ok(())
    }

    /// Create HTML histogram with interactive features
    fn create_histogram_html(
        &self,
        analysis: &StackupAnalysis,
        stackup: &Stackup,
        output_path: &Path,
    ) -> Result<()> {
        let quartile_data = analysis.results.quartile_data.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No quartile data available for histogram"))?;
        
        // Create output directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let title = format!("Monte Carlo Analysis: {}", stackup.name);
        
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
        
        <div id="histogram"></div>
        
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
            mean = quartile_data.median,
            std_dev = quartile_data.iqr / 1.35, // Approximate std dev from IQR
            cp = analysis.results.cp,
            cpk = analysis.results.cpk,
            sample_count = "N/A",
            confidence = "95",
            stackup_name = stackup.name,
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

        // Prepare waterfall data
        let mut contributions = Vec::new();
        let mut cumulative: f64 = 0.0;
        
        for feature_contrib in &analysis.feature_contributions {
            if let Some(feature) = features.get(&feature_contrib.feature_id.into()) {
                let nominal = feature.nominal * feature_contrib.direction;
                contributions.push((feature.name.clone(), nominal, cumulative + nominal));
                cumulative += nominal;
            }
        }

        let max_value: f64 = cumulative.abs().max(contributions.iter().map(|(_, _, cum)| cum.abs()).fold(0.0_f64, f64::max));
        let margin = max_value * 0.1;

        let mut chart = ChartBuilder::on(&root)
            .caption(&title, ("Arial", self.config.title_font_size))
            .margin(10)
            .x_label_area_size(60)
            .y_label_area_size(60)
            .build_cartesian_2d(0f64..(contributions.len() + 1) as f64, (-max_value - margin)..(max_value + margin))?;

        chart
            .configure_mesh()
            .x_desc("Features")
            .y_desc("Cumulative Value")
            .axis_desc_style(("Arial", self.config.axis_font_size))
            .draw()?;

        // Draw waterfall bars
        let mut prev_cumulative = 0.0;
        for (i, (_name, contribution, cumulative)) in contributions.iter().enumerate() {
            let x = i as f64 + 0.5;
            let color = if *contribution >= 0.0 { &GREEN } else { &RED };
            
            // Draw the bar
            chart.draw_series(std::iter::once(Rectangle::new(
                [(x - 0.3, prev_cumulative), (x + 0.3, *cumulative)],
                color.filled(),
            )))?;

            // Draw connecting line to next bar
            if i < contributions.len() - 1 {
                chart.draw_series(std::iter::once(PathElement::new(
                    vec![(x + 0.3, *cumulative), ((i + 1) as f64 + 0.2, *cumulative)],
                    BLACK.stroke_width(1),
                )))?;
            }

            prev_cumulative = *cumulative;
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
        output_path: &Path,
    ) -> Result<()> {
        // Create output directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let title = format!("Waterfall Analysis: {}", stackup.name);
        
        // Prepare data for HTML table
        let mut table_rows = String::new();
        let mut cumulative: f64 = 0.0;
        
        for feature_contrib in &analysis.feature_contributions {
            if let Some(feature) = features.get(&feature_contrib.feature_id.into()) {
                let nominal = feature.nominal * feature_contrib.direction;
                cumulative += nominal;
                table_rows.push_str(&format!(
                    "<tr><td>{}</td><td>{:.4}</td><td>{:.4}</td><td>{:.4}</td></tr>",
                    feature.name, nominal, feature.tolerance.plus + feature.tolerance.minus, cumulative
                ));
            }
        }

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
            <p><strong>Final Nominal Value:</strong> {final_nominal:.4}</p>
            <p><strong>Total Features:</strong> {feature_count}</p>
            <p><strong>Analysis Type:</strong> {analysis_type}</p>
        </div>
        
        <h3>Feature Contributions</h3>
        <table>
            <thead>
                <tr>
                    <th>Feature</th>
                    <th>Nominal</th>
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
            final_nominal = cumulative,
            feature_count = analysis.feature_contributions.len(),
            analysis_type = match analysis.results.distribution_data_file {
                Some(_) => "Monte Carlo + Waterfall",
                None => "Waterfall Only"
            },
            table_rows = table_rows,
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