use std::sync::Arc;
use uuid::Uuid;
use chrono::NaiveDate;
use crate::core::{EdtResult, EdtError};
use crate::storage::RonStorage;
use crate::models::{
    EntityType,
    Assembly, Component, Feature, FeatureType, DistributionType,
    Mate, MateType, Stackup, AnalysisType, Supplier, Quote, CostDistribution,
};
use chrono::Utc;

/// Manages Design, BOM, and Tolerance entities
pub struct DesignManager {
    storage: Arc<RonStorage>,
}

impl DesignManager {
    pub fn new(storage: Arc<RonStorage>) -> Self {
        Self { storage }
    }

    // ============================================================================
    // Assembly Methods
    // ============================================================================

    /// Create an Assembly
    pub fn create_assembly(
        &self,
        name: String,
        description: String,
        revision: String,
    ) -> EdtResult<Assembly> {
        if name.trim().is_empty() {
            return Err(EdtError::ValidationError("Assembly name cannot be empty".to_string()));
        }

        let metadata = crate::models::EntityMetadata::new(EntityType::Assembly);

        let assembly = Assembly {
            metadata,
            name,
            description,
            revision,
            notes: None,
        };

        self.storage.write_assembly(&assembly)?;
        Ok(assembly)
    }

    /// Get an Assembly by ID
    pub fn get_assembly(&self, id: &Uuid) -> EdtResult<Assembly> {
        if !self.storage.exists(&EntityType::Assembly, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.read_assembly(id)
    }

    /// Update an Assembly
    pub fn update_assembly(&self, assembly: Assembly) -> EdtResult<Assembly> {
        if assembly.name.trim().is_empty() {
            return Err(EdtError::ValidationError("Assembly name cannot be empty".to_string()));
        }

        let mut updated = assembly;
        updated.metadata.updated_at = Utc::now();

        self.storage.write_assembly(&updated)?;
        Ok(updated)
    }

    /// Delete an Assembly
    pub fn delete_assembly(&self, id: &Uuid) -> EdtResult<()> {
        if !self.storage.exists(&EntityType::Assembly, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.delete(&EntityType::Assembly, id)
    }

    /// List all Assembly IDs
    pub fn list_assembly_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.storage.list_ids(&EntityType::Assembly)
    }

    // ============================================================================
    // Component Methods
    // ============================================================================

    /// Create a Component
    pub fn create_component(
        &self,
        name: String,
        description: String,
        revision: String,
    ) -> EdtResult<Component> {
        if name.trim().is_empty() {
            return Err(EdtError::ValidationError("Component name cannot be empty".to_string()));
        }

        let metadata = crate::models::EntityMetadata::new(EntityType::Component);

        let component = Component {
            metadata,
            name,
            description,
            revision,
            part_number: None,
            material: None,
            mass: None,
            notes: None,
        };

        self.storage.write_component(&component)?;
        Ok(component)
    }

    /// Get a Component by ID
    pub fn get_component(&self, id: &Uuid) -> EdtResult<Component> {
        if !self.storage.exists(&EntityType::Component, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.read_component(id)
    }

    /// Update a Component
    pub fn update_component(&self, component: Component) -> EdtResult<Component> {
        if component.name.trim().is_empty() {
            return Err(EdtError::ValidationError("Component name cannot be empty".to_string()));
        }

        let mut updated = component;
        updated.metadata.updated_at = Utc::now();

        self.storage.write_component(&updated)?;
        Ok(updated)
    }

    /// Delete a Component
    pub fn delete_component(&self, id: &Uuid) -> EdtResult<()> {
        if !self.storage.exists(&EntityType::Component, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.delete(&EntityType::Component, id)
    }

    /// List all Component IDs
    pub fn list_component_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.storage.list_ids(&EntityType::Component)
    }

    // ============================================================================
    // Feature Methods
    // ============================================================================

    /// Create a Feature
    pub fn create_feature(
        &self,
        name: String,
        description: String,
        feature_type: FeatureType,
        nominal: f64,
        upper_tolerance: f64,
        lower_tolerance: f64,
        distribution_type: DistributionType,
    ) -> EdtResult<Feature> {
        if name.trim().is_empty() {
            return Err(EdtError::ValidationError("Feature name cannot be empty".to_string()));
        }

        if upper_tolerance <= lower_tolerance {
            return Err(EdtError::ValidationError(
                "Upper tolerance must be greater than lower tolerance".to_string()
            ));
        }

        let metadata = crate::models::EntityMetadata::new(EntityType::Feature);

        let feature = Feature {
            metadata,
            name,
            description,
            notes: None,
            feature_type,
            nominal,
            upper_tolerance,
            lower_tolerance,
            distribution_type,
            custom_mean: None,
            custom_std_dev: None,
            drawing_location: None,
        };

        self.storage.write_feature(&feature)?;
        Ok(feature)
    }

    /// Get a Feature by ID
    pub fn get_feature(&self, id: &Uuid) -> EdtResult<Feature> {
        if !self.storage.exists(&EntityType::Feature, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.read_feature(id)
    }

    /// Update a Feature
    pub fn update_feature(&self, feature: Feature) -> EdtResult<Feature> {
        if feature.name.trim().is_empty() {
            return Err(EdtError::ValidationError("Feature name cannot be empty".to_string()));
        }

        if feature.upper_tolerance <= feature.lower_tolerance {
            return Err(EdtError::ValidationError(
                "Upper tolerance must be greater than lower tolerance".to_string()
            ));
        }

        let mut updated = feature;
        updated.metadata.updated_at = Utc::now();

        self.storage.write_feature(&updated)?;
        Ok(updated)
    }

    /// Delete a Feature
    pub fn delete_feature(&self, id: &Uuid) -> EdtResult<()> {
        if !self.storage.exists(&EntityType::Feature, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.delete(&EntityType::Feature, id)
    }

    /// List all Feature IDs
    pub fn list_feature_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.storage.list_ids(&EntityType::Feature)
    }

    // ============================================================================
    // Mate Methods
    // ============================================================================

    /// Create a Mate
    pub fn create_mate(
        &self,
        name: String,
        description: String,
        mate_type: MateType,
    ) -> EdtResult<Mate> {
        if name.trim().is_empty() {
            return Err(EdtError::ValidationError("Mate name cannot be empty".to_string()));
        }

        let metadata = crate::models::EntityMetadata::new(EntityType::Mate);

        let mate = Mate {
            metadata,
            name,
            description,
            notes: None,
            mate_type,
            mmc: None,
            lmc: None,
            analysis_result: None,
        };

        self.storage.write_mate(&mate)?;
        Ok(mate)
    }

    /// Get a Mate by ID
    pub fn get_mate(&self, id: &Uuid) -> EdtResult<Mate> {
        if !self.storage.exists(&EntityType::Mate, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.read_mate(id)
    }

    /// Update a Mate
    pub fn update_mate(&self, mate: Mate) -> EdtResult<Mate> {
        if mate.name.trim().is_empty() {
            return Err(EdtError::ValidationError("Mate name cannot be empty".to_string()));
        }

        let mut updated = mate;
        updated.metadata.updated_at = Utc::now();

        self.storage.write_mate(&updated)?;
        Ok(updated)
    }

    /// Delete a Mate
    pub fn delete_mate(&self, id: &Uuid) -> EdtResult<()> {
        if !self.storage.exists(&EntityType::Mate, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.delete(&EntityType::Mate, id)
    }

    /// List all Mate IDs
    pub fn list_mate_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.storage.list_ids(&EntityType::Mate)
    }

    // ============================================================================
    // Stackup Methods
    // ============================================================================

    /// Create a Stackup
    pub fn create_stackup(
        &self,
        name: String,
        description: String,
        analysis_types: Vec<AnalysisType>,
    ) -> EdtResult<Stackup> {
        if name.trim().is_empty() {
            return Err(EdtError::ValidationError("Stackup name cannot be empty".to_string()));
        }

        if analysis_types.is_empty() {
            return Err(EdtError::ValidationError(
                "At least one analysis type must be specified".to_string()
            ));
        }

        let metadata = crate::models::EntityMetadata::new(EntityType::Stackup);

        let stackup = Stackup {
            metadata,
            name,
            description,
            notes: None,
            analysis_types,
            upper_spec_limit: None,
            lower_spec_limit: None,
            feature_contributions: vec![],
            worst_case_result: None,
            rss_result: None,
            monte_carlo_result: None,
        };

        self.storage.write_stackup(&stackup)?;
        Ok(stackup)
    }

    /// Get a Stackup by ID
    pub fn get_stackup(&self, id: &Uuid) -> EdtResult<Stackup> {
        if !self.storage.exists(&EntityType::Stackup, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.read_stackup(id)
    }

    /// Update a Stackup
    pub fn update_stackup(&self, stackup: Stackup) -> EdtResult<Stackup> {
        if stackup.name.trim().is_empty() {
            return Err(EdtError::ValidationError("Stackup name cannot be empty".to_string()));
        }

        if stackup.analysis_types.is_empty() {
            return Err(EdtError::ValidationError(
                "At least one analysis type must be specified".to_string()
            ));
        }

        let mut updated = stackup;
        updated.metadata.updated_at = Utc::now();

        self.storage.write_stackup(&updated)?;
        Ok(updated)
    }

    /// Delete a Stackup
    pub fn delete_stackup(&self, id: &Uuid) -> EdtResult<()> {
        if !self.storage.exists(&EntityType::Stackup, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.delete(&EntityType::Stackup, id)
    }

    /// List all Stackup IDs
    pub fn list_stackup_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.storage.list_ids(&EntityType::Stackup)
    }

    // ============================================================================
    // Supplier Methods
    // ============================================================================

    /// Create a Supplier
    pub fn create_supplier(
        &self,
        name: String,
        description: String,
    ) -> EdtResult<Supplier> {
        if name.trim().is_empty() {
            return Err(EdtError::ValidationError("Supplier name cannot be empty".to_string()));
        }

        let metadata = crate::models::EntityMetadata::new(EntityType::Supplier);

        let supplier = Supplier {
            metadata,
            name,
            description,
            contact_name: None,
            address: None,
            phone: None,
            email: None,
            notes: None,
        };

        self.storage.write_supplier(&supplier)?;
        Ok(supplier)
    }

    /// Get a Supplier by ID
    pub fn get_supplier(&self, id: &Uuid) -> EdtResult<Supplier> {
        if !self.storage.exists(&EntityType::Supplier, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.read_supplier(id)
    }

    /// Update a Supplier
    pub fn update_supplier(&self, supplier: Supplier) -> EdtResult<Supplier> {
        if supplier.name.trim().is_empty() {
            return Err(EdtError::ValidationError("Supplier name cannot be empty".to_string()));
        }

        let mut updated = supplier;
        updated.metadata.updated_at = Utc::now();

        self.storage.write_supplier(&updated)?;
        Ok(updated)
    }

    /// Delete a Supplier
    pub fn delete_supplier(&self, id: &Uuid) -> EdtResult<()> {
        if !self.storage.exists(&EntityType::Supplier, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.delete(&EntityType::Supplier, id)
    }

    /// List all Supplier IDs
    pub fn list_supplier_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.storage.list_ids(&EntityType::Supplier)
    }

    // ============================================================================
    // Quote Methods
    // ============================================================================

    /// Create a Quote
    pub fn create_quote(
        &self,
        quote_number: String,
        quote_date: NaiveDate,
        quantity_price_pairs: Vec<(u32, f64)>,
        distribution_type: CostDistribution,
    ) -> EdtResult<Quote> {
        if quote_number.trim().is_empty() {
            return Err(EdtError::ValidationError("Quote number cannot be empty".to_string()));
        }

        if quantity_price_pairs.is_empty() {
            return Err(EdtError::ValidationError(
                "At least one quantity-price pair must be provided".to_string()
            ));
        }

        let metadata = crate::models::EntityMetadata::new(EntityType::Quote);

        let quote = Quote {
            metadata,
            quote_number,
            quote_date,
            expiration_date: None,
            quantity_price_pairs,
            distribution_type,
            notes: None,
        };

        self.storage.write_quote(&quote)?;
        Ok(quote)
    }

    /// Get a Quote by ID
    pub fn get_quote(&self, id: &Uuid) -> EdtResult<Quote> {
        if !self.storage.exists(&EntityType::Quote, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.read_quote(id)
    }

    /// Update a Quote
    pub fn update_quote(&self, quote: Quote) -> EdtResult<Quote> {
        if quote.quote_number.trim().is_empty() {
            return Err(EdtError::ValidationError("Quote number cannot be empty".to_string()));
        }

        if quote.quantity_price_pairs.is_empty() {
            return Err(EdtError::ValidationError(
                "At least one quantity-price pair must be provided".to_string()
            ));
        }

        let mut updated = quote;
        updated.metadata.updated_at = Utc::now();

        self.storage.write_quote(&updated)?;
        Ok(updated)
    }

    /// Delete a Quote
    pub fn delete_quote(&self, id: &Uuid) -> EdtResult<()> {
        if !self.storage.exists(&EntityType::Quote, id) {
            return Err(EdtError::EntityNotFound(id.to_string()));
        }
        self.storage.delete(&EntityType::Quote, id)
    }

    /// List all Quote IDs
    pub fn list_quote_ids(&self) -> EdtResult<Vec<Uuid>> {
        self.storage.list_ids(&EntityType::Quote)
    }
}
