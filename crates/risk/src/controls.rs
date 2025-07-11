//! Design control management utilities
//!
//! This module provides utilities for managing design controls and their effectiveness.

use crate::data::*;
use tessera_core::Result;

/// Design control effectiveness analyzer
pub struct ControlEffectivenessAnalyzer;

impl ControlEffectivenessAnalyzer {
    /// Analyze control effectiveness
    pub fn analyze_effectiveness(control: &DesignControl) -> ControlEffectivenessAnalysis {
        let effectiveness_score = control.effectiveness_rating.unwrap_or(0);
        let is_implemented = matches!(control.status, ControlStatus::Implemented | ControlStatus::Verified);
        let is_verified = control.status == ControlStatus::Verified;
        
        let overall_effectiveness = if is_verified {
            effectiveness_score as f64 * 0.2 // Full weight if verified
        } else if is_implemented {
            effectiveness_score as f64 * 0.15 // Reduced weight if not verified
        } else {
            0.0 // No effectiveness if not implemented
        };
        
        ControlEffectivenessAnalysis {
            effectiveness_score: effectiveness_score,
            is_implemented,
            is_verified,
            overall_effectiveness,
            recommendation: Self::generate_recommendation(control),
        }
    }

    /// Generate recommendation for control improvement
    fn generate_recommendation(control: &DesignControl) -> String {
        match control.status {
            ControlStatus::Planned => "Implement the control according to the planned approach".to_string(),
            ControlStatus::InProgress => "Complete the control implementation".to_string(),
            ControlStatus::Implemented => "Verify the control effectiveness".to_string(),
            ControlStatus::Verified => {
                if control.effectiveness_rating.unwrap_or(0) >= 4 {
                    "Control is effective - maintain current approach".to_string()
                } else {
                    "Consider enhancing control effectiveness".to_string()
                }
            },
            ControlStatus::Ineffective => "Redesign or replace the control".to_string(),
        }
    }
}

/// Control effectiveness analysis results
#[derive(Debug, Clone)]
pub struct ControlEffectivenessAnalysis {
    pub effectiveness_score: i32,
    pub is_implemented: bool,
    pub is_verified: bool,
    pub overall_effectiveness: f64,
    pub recommendation: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_effectiveness_analysis() {
        let mut control = DesignControl::new(
            "Test Control".to_string(),
            "A test control".to_string(),
            ControlType::Preventive,
            tessera_core::Id::new(),
        );
        
        control.update_status(ControlStatus::Verified);
        control.set_effectiveness_rating(4);
        
        let analysis = ControlEffectivenessAnalyzer::analyze_effectiveness(&control);
        
        assert_eq!(analysis.effectiveness_score, 4);
        assert!(analysis.is_implemented);
        assert!(analysis.is_verified);
        assert!(analysis.overall_effectiveness > 0.0);
        assert!(!analysis.recommendation.is_empty());
    }
}