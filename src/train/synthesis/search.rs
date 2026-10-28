//! Program search space: enumerates candidate programs and evaluates them.

use super::program::{Program, ProgramOp};
use phiecs::phasor::facet::Facet;

#[derive(Debug, Default)]
pub struct ProgramSearch;

impl ProgramSearch {
    /// Enumerates candidate programs up to a given depth.
    pub fn candidates(depth: usize) -> Vec<Program> {
        let mut programs = vec![Program::identity()];

        let single_ops = vec![
            ProgramOp::Sort,
            ProgramOp::Reverse,
            ProgramOp::Identity,
        ];

        for op in &single_ops {
            programs.push(Program::new(vec![op.clone()]));
        }

        if depth >= 2 {
            for op1 in &single_ops {
                for op2 in &single_ops {
                    if op1 != &ProgramOp::Identity || op2 != &ProgramOp::Identity {
                        programs.push(Program::new(vec![op1.clone(), op2.clone()]));
                    }
                }
            }
        }

        if depth >= 3 {
            for op1 in &single_ops {
                for op2 in &single_ops {
                    for op3 in &single_ops {
                        let ops = vec![op1.clone(), op2.clone(), op3.clone()];
                        if !ops.iter().all(|o| matches!(o, ProgramOp::Identity)) {
                            programs.push(Program::new(ops));
                        }
                    }
                }
            }
        }

        programs
    }

    /// Evaluates how well a program matches input-output examples.
    pub fn evaluate(prog: &Program, _facet: &Facet, examples: &[(String, String)]) -> f64 {
        let mut total = 0.0;
        for (input, expected) in examples {
            let output = prog.execute(input);
            let similarity = Self::string_similarity(&output, expected);
            total += similarity;
        }
        total / examples.len() as f64
    }

    /// Returns the best-matching program from a list of candidates.
    pub fn best(candidates: &[Program], facet: &Facet, examples: &[(String, String)]) -> Option<Program> {
        let mut best: Option<(Program, f64)> = None;
        for prog in candidates {
            let score = Self::evaluate(prog, facet, examples);
            match &best {
                Some((_, best_score)) if score <= *best_score => {}
                _ => best = Some((prog.clone(), score)),
            }
        }
        best.map(|(p, _)| p)
    }

    fn string_similarity(a: &str, b: &str) -> f64 {
        if a == b {
            return 1.0;
        }
        let a_tokens: Vec<&str> = a.split_whitespace().collect();
        let b_tokens: Vec<&str> = b.split_whitespace().collect();
        if a_tokens.is_empty() && b_tokens.is_empty() {
            return 1.0;
        }
        let common = a_tokens.iter().filter(|t| b_tokens.contains(t)).count();
        common as f64 / a_tokens.len().max(b_tokens.len()) as f64
    }
}
