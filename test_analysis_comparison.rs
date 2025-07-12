use tessera_tol::analysis::*;
use tessera_tol::data::*;
use tessera_core::Id;

fn main() {
    // Create two features with normal distributions
    let mut feature1 = Feature::new(
        "Feature1".to_string(),
        "Test feature 1".to_string(),
        Id::new(),
        FeatureType::Length,
        FeatureCategory::External,
        10.0
    );
    feature1.tolerance = Tolerance {
        plus: 0.1,
        minus: 0.1,
        distribution: ToleranceDistribution::Normal,
    };

    let mut feature2 = Feature::new(
        "Feature2".to_string(),
        "Test feature 2".to_string(),
        Id::new(),
        FeatureType::Length,
        FeatureCategory::External,
        20.0
    );
    feature2.tolerance = Tolerance {
        plus: 0.2,
        minus: 0.2,
        distribution: ToleranceDistribution::Normal,
    };

    let features = vec![&feature1, &feature2];

    // RSS Analysis
    let rss_config = AnalysisConfig {
        method: AnalysisMethod::RootSumSquare,
        simulations: 10000,
        confidence_level: 0.95,
    };
    let rss_analyzer = ToleranceAnalyzer::new(rss_config);
    let rss_results = rss_analyzer.root_sum_square_analysis(&features);

    // Monte Carlo Analysis
    let mc_config = AnalysisConfig {
        method: AnalysisMethod::MonteCarlo,
        simulations: 100000, // More samples for better accuracy
        confidence_level: 0.95,
    };
    let mc_analyzer = ToleranceAnalyzer::new(mc_config);
    let mc_results = mc_analyzer.monte_carlo_analysis(&features).unwrap();

    println!("RSS Results:");
    println!("  Nominal: {:.6}", rss_results.nominal_dimension);
    println!("  Tolerance: +{:.6}/-{:.6}", rss_results.predicted_tolerance.plus, rss_results.predicted_tolerance.minus);
    println!("  Total range: {:.6}", rss_results.predicted_tolerance.plus + rss_results.predicted_tolerance.minus);

    println!("\nMonte Carlo Results:");
    println!("  Nominal: {:.6}", mc_results.nominal_dimension);
    println!("  Tolerance: +{:.6}/-{:.6}", mc_results.predicted_tolerance.plus, mc_results.predicted_tolerance.minus);
    println!("  Total range: {:.6}", mc_results.predicted_tolerance.plus + mc_results.predicted_tolerance.minus);

    let range_difference = (rss_results.predicted_tolerance.plus + rss_results.predicted_tolerance.minus) - 
                          (mc_results.predicted_tolerance.plus + mc_results.predicted_tolerance.minus);
    let range_difference_percent = (range_difference / (rss_results.predicted_tolerance.plus + rss_results.predicted_tolerance.minus)) * 100.0;

    println!("\nComparison:");
    println!("  Range difference: {:.6} ({:.2}%)", range_difference, range_difference_percent);
    
    if range_difference_percent.abs() < 5.0 {
        println!("  ✅ Results are within 5% - GOOD!");
    } else {
        println!("  ❌ Results differ by more than 5% - ISSUE!");
    }

    // Theoretical calculation for verification
    let feature1_std = (feature1.tolerance.plus + feature1.tolerance.minus) / 6.0;
    let feature2_std = (feature2.tolerance.plus + feature2.tolerance.minus) / 6.0;
    let total_variance = feature1_std.powi(2) + feature2_std.powi(2);
    let total_std = total_variance.sqrt();
    let theoretical_3sigma = 3.0 * total_std;

    println!("\nTheoretical (manual calculation):");
    println!("  Feature1 std dev: {:.6}", feature1_std);
    println!("  Feature2 std dev: {:.6}", feature2_std);
    println!("  Total std dev: {:.6}", total_std);
    println!("  3-sigma range: {:.6}", 2.0 * theoretical_3sigma);
}