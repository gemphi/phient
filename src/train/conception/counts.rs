#[allow(unused_imports)]
use super::*;

impl DefinitionGraph {

    /// `(strong, weak)` edge counts. Reported so the ratio can be sanity-checked
    /// against the source: dict2vec extracted 417K strong to 3.9M weak, roughly
    /// 1:9, from 200K definitions.
    pub fn counts(&self) -> (usize, usize) {
        let (mut strong, mut weak) = (0usize, 0usize);
        for (a, defs) in &self.edges {
            for b in defs {
                match self.contains(b, a) {
                    true => strong += 1,
                    false => weak += 1,
                }
            }
        }
        (strong / 2, weak)
    }
}
