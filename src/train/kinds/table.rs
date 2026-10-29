#[allow(unused_imports)]
use super::*;
use phical::circle::{self, Resultant};

impl OffsetTable {
    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    /// Computes every pair's offset under `est`.
    pub fn build(
        facet: &Facet,
        pairs: &[(String, String)],
        est: OffsetEstimator,
    ) -> Self {
        let mut kept: Vec<(String, String)> = Vec::new();
        let mut raw: Vec<Vec<f64>> = Vec::new();
        for p in pairs {
            if let Some(o) = RoleDiscovery::offset(facet, &p.0, &p.1) {
                kept.push(p.clone());
                raw.push(o);
            }
        }
        if matches!(est, OffsetEstimator::Raw) || kept.is_empty() {
            return Self { pairs: kept, offsets: raw };
        }

        // Circular means, accumulated as unit vectors. A mean of angles is the
        // argument of the sum of their unit vectors, not the mean of the angles.
        let mut head_acc: HashMap<&str, Vec<Resultant>> = HashMap::new();
        let mut filler_acc: HashMap<&str, Vec<Resultant>> = HashMap::new();
        let mut grand = vec![Resultant::new(); PHASE_CHANNELS];
        let zeros = || vec![Resultant::new(); PHASE_CHANNELS];

        for (p, o) in kept.iter().zip(raw.iter()) {
            let h = head_acc.entry(p.0.as_str()).or_insert_with(zeros);
            for k in 0..PHASE_CHANNELS {
                h[k].add_unit(o[k]);
                grand[k].add_unit(o[k]);
            }
            let f = filler_acc.entry(p.1.as_str()).or_insert_with(zeros);
            for k in 0..PHASE_CHANNELS {
                f[k].add_unit(o[k]);
            }
        }

        let arg_or_zero = |r: Resultant| r.direction_or_none().unwrap_or(0.0);

        let offsets: Vec<Vec<f64>> = kept
            .iter()
            .zip(raw.iter())
            .map(|(p, o)| {
                let h = &head_acc[p.0.as_str()];
                let f = &filler_acc[p.1.as_str()];
                (0..PHASE_CHANNELS)
                    .map(|k| match est {
                        OffsetEstimator::Raw => circle::reduce(o[k]),
                        OffsetEstimator::HeadCentered => circle::centred(o[k], arg_or_zero(h[k])),
                        OffsetEstimator::DoubleCentered => circle::double_centred(
                            o[k],
                            arg_or_zero(h[k]),
                            arg_or_zero(f[k]),
                            arg_or_zero(grand[k]),
                        ),
                    })
                    .collect()
            })
            .collect();

        Self { pairs: kept, offsets }
    }

    /// Offsets centered within a *context class* — Searle's C.
    ///
    /// Searle's constitutive rule is "X counts as Y **in C**", and this codebase
    /// has been dropping the third term: every relation measured here is X to Y
    /// with no context, and `roles::Role::Context` is declared and never once
    /// assigned. A context-free offset is not a partial version of the rule, it
    /// is a different claim — that the step from X to Y means the same thing
    /// everywhere — and head domination at 59.8% against a 9.1% floor is what
    /// that claim looks like when it fails.
    ///
    /// Here C enters as a frame that shifts everything inside it: the mean offset
    /// of the class is subtracted from each of its members, so what remains is
    /// what this pair does *that its context does not already do*. A relation
    /// that is only a property of the context vanishes; one that survives is a
    /// relation the context does not explain.
    ///
    /// `classes` is one class per pair, in the same order as `pairs`. Where it
    /// comes from is the caller's problem and the experiment's variable — a
    /// syntactic classifier, a clustering of the sentences, or a shuffle with
    /// matching class sizes, which is the control that decides whether C is doing
    /// anything at all.
    pub fn build_contextual(
        facet: &Facet,
        pairs: &[(String, String)],
        classes: &[u16],
    ) -> Self {
        let mut kept: Vec<(String, String)> = Vec::new();
        let mut raw: Vec<Vec<f64>> = Vec::new();
        let mut cls: Vec<u16> = Vec::new();
        for (i, p) in pairs.iter().enumerate() {
            if let Some(o) = RoleDiscovery::offset(facet, &p.0, &p.1) {
                kept.push(p.clone());
                raw.push(o);
                cls.push(*classes.get(i).unwrap_or(&0));
            }
        }
        if kept.is_empty() {
            return Self { pairs: kept, offsets: raw };
        }

        let mut acc: HashMap<u16, Vec<Resultant>> = HashMap::new();
        for (c, o) in cls.iter().zip(raw.iter()) {
            let e = acc.entry(*c).or_insert_with(|| vec![Resultant::new(); PHASE_CHANNELS]);
            for k in 0..PHASE_CHANNELS {
                e[k].add_unit(o[k]);
            }
        }

        let offsets: Vec<Vec<f64>> = raw
            .iter()
            .zip(cls.iter())
            .map(|(o, c)| {
                let m = &acc[c];
                (0..PHASE_CHANNELS)
                    .map(|k| circle::centred(o[k], m[k].direction_or_none().unwrap_or(0.0)))
                    .collect()
            })
            .collect();

        Self { pairs: kept, offsets }
    }

    /// Held-out agreement of these offsets to the fixed harmonic basis.
    ///
    /// Nothing is fitted, so every pair is held out and the comparison across
    /// context sources is a comparison of the contexts alone.
    pub fn harmonic_agreement(&self) -> f64 {
        circle::mean_nearest(&self.offsets, &KindTable::harmonic().rotations)
    }
}
