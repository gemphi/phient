#[allow(unused_imports)]
use super::*;

/// Encodes a phasor as a unit-energy function on the relation circle.
pub fn encode(p: &SpectralPhasor) -> WordFunction {
    encode_side(p, false)
}

/// Encodes one side of a phasor as a unit-energy function.
pub fn encode_side(p: &SpectralPhasor, target: bool) -> WordFunction {
    let mut f = vec![c64::new(0.0, 0.0); PHASE_CHANNELS];
    let evidenced = p.has_evidence();
    let mut energy = 0.0;
    for (k, v) in f.iter_mut().enumerate() {
        let a = match evidenced {
            true => p.evidence(k),
            false => 1.0,
        };
        let th = if target { p.theta_target(k) } else { p.theta(k) };
        *v = c64::from_polar(a, th);
        energy += a * a;
    }
    let scale = 1.0 / energy.sqrt().max(1e-12);
    for v in f.iter_mut() {
        *v *= scale;
    }
    f
}

/// Both sides, source then target, as one 128-sample function.
pub fn encode_both(p: &SpectralPhasor) -> WordFunction {
    let mut f = encode_side(p, false);
    f.extend(encode_side(p, true));
    normalise(&mut f);
    f
}

impl FunctionSpace {
    /// Encodes every word in the facet. Word order is sorted, so the space is
    /// identical across runs whatever the lexicon's hash-map order.
    pub fn from_facet(facet: &Facet) -> Self {
        Self::from_facet_sides(facet, Sides::Source)
    }

    /// Encodes every word with the chosen side(s).
    pub fn from_facet_sides(facet: &Facet, sides: Sides) -> Self {
        let mut words: Vec<&String> = facet.lexicon.keys().collect();
        words.sort();
        let mut index = HashMap::with_capacity(words.len());
        let mut funcs = Vec::with_capacity(words.len());
        let mut table = Vec::new();
        let mut counts = Vec::with_capacity(words.len());
        let mut evidenced = false;
        for (i, w) in words.iter().enumerate() {
            let p = &facet.lexicon[*w];
            evidenced |= p.has_evidence();
            index.insert((*w).clone(), i);
            funcs.push(match sides {
                Sides::Source | Sides::Directed => encode_side(p, false),
                Sides::Target => encode_side(p, true),
                Sides::Both => encode_both(p),
            });
            if sides == Sides::Directed {
                table.push(encode_side(p, true));
            }
            counts.push(p.count);
        }
        let table = match sides {
            Sides::Directed => Some(table),
            _ => None,
        };
        Self { words: words.into_iter().cloned().collect(), index, funcs, counts, evidenced, table }
    }

    /// The functions words are ranked against (see [`Self::table`]).
    pub fn table(&self) -> &[WordFunction] {
        self.table.as_deref().unwrap_or(&self.funcs)
    }

    /// Samples per function.
    pub fn space(&self) -> usize {
        self.funcs.first().map_or(PHASE_CHANNELS, |f| f.len())
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    pub fn get(&self, word: &str) -> Option<&WordFunction> {
        self.index.get(word).map(|i| &self.funcs[*i])
    }

    /// A context as a function: the mean of the preceding `window` words'
    /// functions, nearer words weighted more, normalised.
    ///
    /// This is the superposition the engine already uses for a context wave —
    /// the input the operator will be asked to improve on, and the identity
    /// baseline it is measured against.
    pub fn context(&self, toks: &[usize], end: usize, window: usize) -> Option<WordFunction> {
        let start = end.saturating_sub(window);
        let mut f = vec![c64::new(0.0, 0.0); self.space()];
        let mut used = 0;
        for (j, &t) in toks[start..end].iter().enumerate() {
            let dist = end - (start + j);
            let w = 1.0 / dist as f64;
            for (k, v) in f.iter_mut().enumerate() {
                *v += self.funcs[t][k] * w;
            }
            used += 1;
        }
        match used > 0 && normalise(&mut f) {
            true => Some(f),
            false => None,
        }
    }

    /// Words with at least `floor` observations, as a sorted candidate pool.
    pub fn pool(&self, floor: u32) -> Vec<usize> {
        (0..self.len()).filter(|&i| self.counts[i] >= floor).collect()
    }
}

/// Builds `(context, next)` examples from tokenised sentences.
///
/// Only positions whose next word is in `pool` are kept, so the ranking task
/// and the training task see the same candidate set. A deterministic hash of
/// the sentence puts it in the held-out set with probability 1/`holdout_every`.
pub fn examples(
    space: &FunctionSpace,
    sentences: &[Vec<String>],
    window: usize,
    pool: &[usize],
    holdout_every: u64,
) -> (Vec<Example>, Vec<Example>) {
    let in_pool: std::collections::HashSet<usize> = pool.iter().copied().collect();
    let (mut train, mut held) = (Vec::new(), Vec::new());
    for s in sentences {
        let toks: Vec<usize> = s.iter().filter_map(|w| space.index.get(w).copied()).collect();
        if toks.len() < 2 {
            continue;
        }
        let held_out = fnv1a(&s.join(" ")).is_multiple_of(holdout_every);
        for end in 1..toks.len() {
            let next = toks[end];
            if !in_pool.contains(&next) {
                continue;
            }
            if let Some(context) = space.context(&toks, end, window) {
                let ex = Example { input: context, target: next };
                match held_out {
                    true => held.push(ex),
                    false => train.push(ex),
                }
            }
        }
    }
    (train, held)
}
