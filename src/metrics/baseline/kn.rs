//! The Kneser-Ney baseline, cut with phiano's own [`Tokenizer`] so its
//! perplexity is on the same tokens the facet reads. The counting is
//! [`crate::baseline::KneserNey`] (docs/159).

use phiton::p64::words::Tokenizer;

/// Interpolated Kneser-Ney trigram model over phiano-tokenized sentences.
#[derive(Debug, Default)]
pub struct KneserNey(crate::baseline::KneserNey);

fn cut(corpus: &[String]) -> Vec<Vec<String>> {
    corpus.iter().map(|s| Tokenizer::tokenize(s)).collect()
}

impl KneserNey {
    /// Builds the model from a corpus of sentences.
    pub fn train(corpus: &[String]) -> Self {
        Self(crate::baseline::KneserNey::train(&cut(corpus)))
    }

    /// Interpolated KN trigram probability, backing off to bigram then continuation.
    pub fn probability(&self, a: &str, b: &str, c: &str) -> f64 {
        self.0.probability(a, b, c)
    }

    /// Held-out perplexity: exp(-mean log probability per token).
    pub fn perplexity(&self, held_out: &[String]) -> f64 {
        self.0.perplexity(&cut(held_out))
    }

    pub fn vocabulary_size(&self) -> usize {
        self.0.vocabulary_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shim_tokenizes_as_the_facet_does() {
        let corpus = vec!["The cat sat on the mat.".to_string()];
        let m = KneserNey::train(&corpus);
        let distinct = Tokenizer::tokenize(&corpus[0]).into_iter().collect::<std::collections::HashSet<_>>().len();
        assert_eq!(m.vocabulary_size(), distinct + 2, "the tokens plus <s> and </s>");
        assert!(m.perplexity(&["a different sentence".to_string()]).is_finite());
    }
}
