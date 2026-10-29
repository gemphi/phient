#[allow(unused_imports)]
use super::*;

impl Evaluator {
    /// Creates a new composer evaluator.
    pub fn new() -> Self {
        Self { inner: crate::eval::Evaluator::new() }
    }

    /// Assigns a composition its coarse subgoal stage.
    ///
    /// Stages are disjoint and ordered. Only within a stage does the fine
    /// measure decide, so progress on the coarse goal can never be sold for
    /// diversity or length.
    ///
    /// * 0 — degenerate: too short, or the wave cancels to nothing
    /// * 1 — off-prompt: shares no content word with the prompt
    /// * 2 — weakly on-prompt: shares vocabulary, but the waves barely align
    /// * 3 — on-prompt
    pub fn stage(facet: &Facet, prompt: &str, text: &str) -> u8 {
        let toks = Tokenizer::tokenize(text);
        let content = Tokenizer::content_words(text);
        if content.len() < 3 {
            return 0;
        }
        let w = Wave::sentence(facet, &toks);
        if w.norm() < 1e-6 {
            return 0;
        }

        let prompt_content: HashSet<String> = Tokenizer::content_words(prompt).into_iter().collect();
        let shares = content.iter().any(|c| prompt_content.contains(c));
        let align = Self::alignment(facet, prompt, &toks);

        match (shares, align) {
            (false, a) if a < 0.55 => 1,
            (_, a) if a < 0.70 => 2,
            _ => 3,
        }
    }

    /// Scores a composition using a **fixed** facet as the referee.
    ///
    /// Samuel's Alpha never promoted itself: "better" meant winning games
    /// against a frozen Beta, so the evaluator being trained was never allowed
    /// to referee its own promotion. Scoring a round's output against the
    /// phases as they stood *before* that round is the same discipline — the
    /// prompt and the frozen manifold are both outside the thing being judged.
    pub fn referee_score(frozen: &Facet, prompt: &str, text: &str) -> f64 {
        let tokens = Tokenizer::tokenize(text);
        if Tokenizer::content_words(text).len() < 3 {
            return 0.0;
        }
        let stage = Self::stage(frozen, prompt, text) as f64;
        let align = Self::alignment(frozen, prompt, &tokens);
        // Stage dominates, alignment breaks ties — the same lexicographic order
        // the tournament itself uses.
        stage + align
    }

    /// Cosine alignment between the prompt wave and a composition's wave,
    /// mapped from [-1, 1] to [0, 1].
    pub(super) fn alignment(facet: &Facet, prompt: &str, tokens: &[String]) -> f64 {
        let pw = Wave::text(facet, prompt);
        let cw = Wave::sentence(facet, tokens);
        let (pn, cn) = (pw.norm(), cw.norm());
        if pn <= 0.0 || cn <= 0.0 {
            return 0.0;
        }
        (((pw.re * cw.re + pw.im * cw.im) / (pn * cn)) + 1.0) / 2.0
    }

    /// Evaluates all sector variations and returns them ranked, best first.
    ///
    /// Ranking is lexicographic on `(stage, score)`. `score` itself is a
    /// normalised weighted sum — normalised because the raw coefficients sum to
    /// 1.15, and because `length_factor` previously borrowed `WEIGHT_NOVELTY`,
    /// counting novelty twice and leaving length without a weight of its own.
    pub fn evaluate_variations(
        &self,
        facet: &Facet,
        flows: &[RiverFlow],
        killers: &KillerWord,
    ) -> Vec<SectorScore> {
        let mut scores = Vec::with_capacity(flows.len());
        let prompt = flows.first().map(|f| f.prompt.as_str()).unwrap_or("");

        let base_weight =
            config::WEIGHT_COHERENCE + config::WEIGHT_NOVELTY + config::WEIGHT_RESONANCE;
        let total_weight = base_weight
            + config::WEIGHT_DIVERSITY
            + config::WEIGHT_COVERAGE
            + config::WEIGHT_LENGTH
            + config::WEIGHT_ALIGNMENT;

        for flow in flows {
            let eval = self.inner.eval(facet, &flow.text);
            let tokens = Tokenizer::tokenize(&flow.text);

            let unique: HashSet<&String> = tokens.iter().collect();
            let diversity = if tokens.is_empty() {
                0.0
            } else {
                unique.len() as f64 / tokens.len() as f64
            };

            let mut sectors_used = HashSet::new();
            for token in &tokens {
                if let Some(s) = Wave::word_sector(facet, token) {
                    sectors_used.insert(s);
                }
            }
            let coverage = sectors_used.len() as f64 / Wave::sector_count() as f64;

            let length_factor = if tokens.len() < 10 {
                tokens.len() as f64 / 10.0
            } else {
                1.0
            };

            let alignment = Self::alignment(facet, prompt, &tokens);

            let raw = eval.overall * base_weight
                + diversity * config::WEIGHT_DIVERSITY
                + coverage * config::WEIGHT_COVERAGE
                + length_factor * config::WEIGHT_LENGTH
                + alignment * config::WEIGHT_ALIGNMENT;

            let measure = ((raw / total_weight) * killers.bonus(&flow.text)).clamp(0.0, 1.0);

            scores.push(SectorScore {
                sector: flow.source_sector,
                color: SectorPalette::color(flow.source_sector),
                stage: Self::stage(facet, prompt, &flow.text),
                text: flow.text.clone(),
                score: measure,
            });
        }

        // Lexicographic: stage first, measure only as a tie-break.
        scores.sort_by(|a, b| {
            b.stage
                .cmp(&a.stage)
                .then(b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal))
        });

        scores
    }

    /// Computes the average measure across all sectors.
    pub fn average_score(&self, scores: &[SectorScore]) -> f64 {
        phistd::stats::mean(&scores.iter().map(|s| s.score).collect::<Vec<_>>())
    }

    /// Standard deviation of sector measures.
    ///
    /// This is the manifold-health readout of the tournament. A spread trending
    /// to zero means the variants have become indistinguishable — there is
    /// nothing left to select between — which looks identical to convergence
    /// from the top score alone.
    pub fn score_spread(&self, scores: &[SectorScore]) -> f64 {
        phistd::stats::population_sd(&scores.iter().map(|s| s.score).collect::<Vec<_>>())
    }

    /// True when the variant population has collapsed rather than converged.
    pub fn is_degenerate(&self, scores: &[SectorScore]) -> bool {
        self.score_spread(scores) < config::SPREAD_ALARM
    }
}
