use crate::classifier::Classifier;
use crate::findings::Finding;
use crate::rules::builtin::TextClassifier;

pub struct Scanner {
    classifiers: Vec<Box<dyn Classifier>>,
}

impl Scanner {
    pub fn new() -> Self {
        Self {
            classifiers: Vec::new(),
        }
    }

    pub fn with_defaults() -> Self {
        let mut scanner = Self::new();
        scanner.add_classifier(Box::new(TextClassifier::new()));
        scanner
    }

    pub fn add_classifier(&mut self, classifier: Box<dyn Classifier>) {
        self.classifiers.push(classifier);
    }

    pub fn classifier_count(&self) -> usize {
        self.classifiers.len()
    }

    pub fn scan(&self, input: &str) -> Vec<Finding> {
        self.classifiers
            .iter()
            .flat_map(|c| c.classify(input))
            .collect()
    }
}

impl Default for Scanner {
    fn default() -> Self {
        Self::with_defaults()
    }
}
