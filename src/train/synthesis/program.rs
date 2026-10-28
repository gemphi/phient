//! Program representation and synthesis core.

use phiecs::phasor::facet::Facet;
use phiton::p64::words::Tokenizer;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ProgramOp {
    Map(String),
    Filter(String),
    Reduce(String, String),
    Compose,
    Sort,
    Reverse,
    Identity,
}

#[derive(Debug, Clone, Serialize)]
pub struct Program {
    pub operations: Vec<ProgramOp>,
    pub phase_template: Vec<f64>,
}

impl Program {
    pub fn new(ops: Vec<ProgramOp>) -> Self {
        Self { operations: ops, phase_template: Vec::new() }
    }

    pub fn identity() -> Self {
        Self::new(vec![ProgramOp::Identity])
    }

    /// Executes the program on an input string.
    pub fn execute(&self, input: &str) -> String {
        let mut tokens = Tokenizer::tokenize(input);
        for op in &self.operations {
            tokens = match op {
                ProgramOp::Sort => {
                    let mut t = tokens.clone();
                    t.sort();
                    t
                }
                ProgramOp::Reverse => {
                    tokens.iter().rev().cloned().collect()
                }
                ProgramOp::Identity => tokens.clone(),
                ProgramOp::Map(_) => tokens.to_vec(),
                ProgramOp::Filter(word) => {
                    tokens.iter().filter(|t| *t != word).cloned().collect()
                }
                _ => tokens.clone(),
            };
        }
        tokens.join(" ")
    }
}

#[derive(Debug, Default)]
pub struct ProgramSynthesizer;

impl ProgramSynthesizer {
    /// Attempts to synthesize a program from input-output examples.
    pub fn synthesize(facet: &Facet, examples: &[(String, String)]) -> Option<Program> {
        if examples.is_empty() {
            return None;
        }

        let candidates = super::search::ProgramSearch::candidates(3);
        super::search::ProgramSearch::best(&candidates, facet, examples)
    }
}
