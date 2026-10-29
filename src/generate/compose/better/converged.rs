#[allow(unused_imports)]
use super::*;

impl Evaluator {

    /// Checks whether the tournament has converged on the top score.
    pub fn has_converged(&self, scores: &[SectorScore], prev_best: f64) -> bool {
        match scores.first() {
            None => true,
            Some(top) => (top.score - prev_best).abs() < config::COMPOSE_CONVERGENCE_DELTA,
        }
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}
