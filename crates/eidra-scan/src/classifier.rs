use crate::findings::Finding;

/// Trait for all data classifiers.
pub trait Classifier: Send + Sync {
    fn classify(&self, input: &str) -> Vec<Finding>;
    fn name(&self) -> &str;
}
