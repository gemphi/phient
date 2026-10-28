#[allow(unused_imports)]
use super::*;

impl OffsetEstimator {
    pub fn label(self) -> &'static str {
        match self {
            OffsetEstimator::Raw => "raw",
            OffsetEstimator::HeadCentered => "head-centered",
            OffsetEstimator::DoubleCentered => "double-centered",
        }
    }

    pub const ALL: [OffsetEstimator; 3] = [
        OffsetEstimator::Raw,
        OffsetEstimator::HeadCentered,
        OffsetEstimator::DoubleCentered,
    ];
}
