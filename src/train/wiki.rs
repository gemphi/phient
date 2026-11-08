#![allow(dead_code)]

//! Wikipedia bulk ingestion - downloads and trains on multiple Wikipedia
//! articles in sequence for large-scale knowledge acquisition.

mod extracts;
#[allow(unused_imports)]
use extracts::*;

use super::chunker::ChunkStore;
use phiecs::phasor::facet::Facet;
use phiton::p64::words::Tokenizer;
use super::trainer::Trainer;

/// A curated set of topics for comprehensive knowledge ingestion.
/// Organized by domain for structured learning.
pub const CURRICULUM_TOPICS: &[&[&str]] = &[
    // Science
    &[
        "Physics", "Chemistry", "Biology", "Astronomy", "Geology",
        "Mathematics", "Statistics", "Calculus", "Algebra", "Geometry",
        "Quantum mechanics", "Relativity", "Thermodynamics", "Entropy",
        "Evolution", "Genetics", "DNA", "Cell (biology)", "Protein",
        "Photosynthesis", "Ecosystem", "Climate change", "Atom",
    ],
    // Technology
    &[
        "Computer science", "Programming language", "Algorithm",
        "Data structure", "Operating system", "Database",
        "Internet", "World Wide Web", "Cryptography", "Artificial intelligence",
        "Machine learning", "Deep learning", "Neural network",
        "Transformer (machine learning model)", "Attention mechanism",
        "Backpropagation", "Gradient descent", "Natural language processing",
        "Computer vision", "Reinforcement learning",
    ],
    // Philosophy
    &[
        "Philosophy", "Epistemology", "Metaphysics", "Ethics", "Logic",
        "Awareness", "Intentionality", "Phenomenology",
        "Existentialism", "Pragmatism", "Rationalism", "Empiricism",
        "Philosophy of mind", "Philosophy of language", "Aesthetics",
    ],
    // History & Society
    &[
        "History", "Civilization", "Democracy", "Capitalism",
        "Socialism", "Revolution", "Colonialism", "World War I",
        "World War II", "Cold War", "Renaissance", "Enlightenment",
        "Industrial Revolution", "Information Age", "Globalization",
    ],
    // Arts & Literature
    &[
        "Literature", "Poetry", "Novel", "Fiction", "Mythology",
        "Music", "Painting", "Sculpture", "Architecture", "Theatre",
        "Film", "Photography", "Dance", "Opera",
    ],
];

/// Result of a bulk ingestion run.
#[derive(Debug, Clone, serde::Serialize)]
pub struct IngestionResult {
    pub topics_attempted: usize,
    pub topics_succeeded: usize,
    pub total_tokens_trained: usize,
    pub vocabulary_before: usize,
    pub vocabulary_after: usize,
    pub errors: Vec<String>,
}

/// Wikipedia bulk ingestion operations.
pub struct WikiBulk;
