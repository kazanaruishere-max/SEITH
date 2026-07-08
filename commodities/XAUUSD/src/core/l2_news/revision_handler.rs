// L2 - Revision Handler
// Detect manipulation of previous month's data (Revised vs Original)

use super::data_extractor::ExtractedData;

#[derive(Debug, Clone)]
pub struct RevisionInfo {
    pub original: f64,
    pub revised: f64,
    pub revision_delta: f64,
    pub has_revision: bool,
}

impl RevisionInfo {
    pub fn new(original: f64, revised: f64) -> Self {
        Self {
            original,
            revised,
            revision_delta: revised - original,
            has_revision: (revised - original).abs() > f64::EPSILON,
        }
    }
}

pub fn detect_revision(data: &ExtractedData) -> RevisionInfo {
    RevisionInfo::new(data.original_previous, data.revised_previous)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_revision() {
        let info = RevisionInfo::new(1.0, 1.0);
        assert!(!info.has_revision);
    }

    #[test]
    fn test_revision_exists() {
        let info = RevisionInfo::new(1.0, 0.5);
        assert!(info.has_revision);
        assert!((info.revision_delta + 0.5).abs() < 0.01);
    }
}
