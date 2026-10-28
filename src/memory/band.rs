#[allow(unused_imports)]
use super::*;

impl MemoryBand {
    /// Returns the starting layer index for this band (0, 4, 8, or 12).
    pub fn base_layer(self) -> usize {
        match self {
            Self::Surface => 0,
            Self::Pattern => 4,
            Self::Semantic => 8,
            Self::Deep => 12,
        }
    }

    /// Returns the band that contains the given layer index.
    pub fn from_layer(layer: usize) -> Self {
        match layer / 4 {
            0 => Self::Surface,
            1 => Self::Pattern,
            2 => Self::Semantic,
            _ => Self::Deep,
        }
    }
}

impl fmt::Display for MemoryBand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Surface => write!(f, "surface"),
            Self::Pattern => write!(f, "pattern"),
            Self::Semantic => write!(f, "semantic"),
            Self::Deep => write!(f, "deep"),
        }
    }
}
