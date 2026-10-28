//! Building identities from a composition, recovering fillers, and the cleanup examples.

use super::*;

impl Composition {
    /// highest available level.
    pub fn identities(&self, space: &FunctionSpace, level: usize) -> HashMap<usize, WordFunction> {
        let mut cur: HashMap<usize, WordFunction> = self.words.iter().map(|&w| (w, space.funcs[w].clone())).collect();
        for _ in 0..level {
            let mut next = HashMap::with_capacity(cur.len());
            for &w in &self.words {
                let parts: Vec<WordFunction> = self.fillers[&w]
                    .iter()
                    .map(|&(k, filler_w)| {
                        let filler = cur.get(&filler_w).cloned().unwrap_or_else(|| space.funcs[filler_w].clone());
                        bind(&filler, &self.roles[k])
                    })
                    .collect();
                next.insert(w, compose(&parts));
            }
            cur = next;
        }
        cur
    }

    /// Recovery at `level`: for each composable word and role, unbind and rank
    /// the true level-`(level−1)` object among all of them. Returns
    /// `(MRR, hit@1)`, with `cleanup` applied to the unbound query if given.
    pub fn recover(
        &self,
        space: &FunctionSpace,
        level: usize,
        words: &[usize],
        cleanup: Option<&mut Fno>,
    ) -> (f64, f64) {
        assert!(level >= 1);
        let top = self.identities(space, level);
        let below = self.identities(space, level - 1);
        let cands: Vec<(usize, WordFunction)> = self.words.iter().map(|&w| (w, below[&w].clone())).collect();

        let mut queries: Vec<(WordFunction, usize)> = Vec::new();
        for &w in words {
            for &(k, filler) in &self.fillers[&w] {
                queries.push((unbind(&top[&w], &self.roles[k]), filler));
            }
        }
        let inputs: Vec<WordFunction> = queries.iter().map(|q| q.0.clone()).collect();
        let cleaned: Vec<WordFunction> = match cleanup {
            Some(fno) => fno.apply(&inputs),
            None => inputs,
        };
        let (mut mrr, mut hit) = (0.0, 0usize);
        for ((_, target), q) in queries.iter().zip(&cleaned) {
            let t = match below.get(target) {
                Some(t) => cosine(q, t),
                None => continue,
            };
            let above = cands.iter().filter(|(w, c)| w != target && cosine(q, c) > t).count();
            mrr += 1.0 / (1 + above) as f64;
            if above == 0 {
                hit += 1;
            }
        }
        let n = queries.len().max(1) as f64;
        (mrr / n, hit as f64 / n)
    }

    /// Training pairs for the cleanup at `level`: `(unbind(I_level(w), d) →
    /// filler_d)`, where the target is the filler's level-`(level−1)` object.
    /// Returns the pairs and a function space whose entries *are* those
    /// objects, so the pairs can be fed to `Fno::train_epoch` unchanged.
    ///
    /// A cleanup trained on level 1 alone learns to map anything toward a
    /// plain word function, and at level 2 — where the answer is an identity,
    /// not a word — it measured *worse* than no cleanup (0.059 against 0.086
    /// MRR). The recursion needs a cleanup that has seen every level it will
    /// be asked to clean.
    pub fn cleanup_examples(
        &self,
        space: &FunctionSpace,
        words: &[usize],
        level: usize,
    ) -> (Vec<(WordFunction, usize)>, FunctionSpace) {
        let top = self.identities(space, level.max(1));
        let below = self.identities(space, level.max(1) - 1);
        let mut targets = FunctionSpace {
            words: space.words.clone(),
            index: space.index.clone(),
            funcs: space.funcs.clone(),
            counts: space.counts.clone(),
            evidenced: space.evidenced,
            table: None,
        };
        for (&w, f) in &below {
            targets.funcs[w] = f.clone();
        }
        let mut out = Vec::new();
        for &w in words {
            for &(k, filler) in &self.fillers[&w] {
                out.push((unbind(&top[&w], &self.roles[k]), filler));
            }
        }
        (out, targets)
    }
}
