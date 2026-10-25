//! The cut: a story divided, most informative division first, until every
//! word stands alone. To divide a span is to say it was two segments, not
//! one, and that is worth saying when it shortens the story's description:
//! the span's code length as one segment, minus the two halves' as two,
//! minus what it costs to say the division (where it falls, `ln(n − 1)`,
//! and the two tree flags it adds, `2 ln 2`). A segment's code length is
//! its tokens predicted in order with Witten–Bell escape (PPM-C), centred on
//! the parent's distribution — the same breadth smoothing the levels use
//! (docs/131): `p(t) = (c(t) + T · p_parent(t)) / (i + T)` with `T` the
//! types seen so far. Information maximisation and minimum description are
//! one criterion here.
//!
//! Every span's best division depends only on the span and its parent, so
//! the whole tree is built first, down to the word. A division is then
//! worth making if it, together with the best of what it unlocks below,
//! shortens the description (`value`, MDL pruning bottom-up): the story's
//! information supports the pruned tree's leaves. The order divisions are
//! made in — which the sixty-four levels are read from — is the pruned
//! tree's first, most information unlocked first; below it, where no
//! division pays, the largest unit is divided first at its best cut.

use super::code::share_of;
use std::collections::HashMap;

/// A span of tokens `[start, end)` and its place in the history.
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub parent: Option<usize>,
    /// The step at which it was born (0 for the whole story).
    pub born: usize,
    /// The step at which it was divided, if it was.
    pub divided: Option<usize>,
    /// Its word counts.
    pub counts: HashMap<u32, u32>,
    /// Its Witten–Bell distribution over its own words, smoothed toward its parent's.
    pub share: HashMap<u32, f64>,
    /// Its best division (`at`, net gain) and the two spans it makes.
    pub division: Option<(usize, f64)>,
    pub children: Option<(usize, usize)>,
    /// What dividing it is worth with the best of what that unlocks, if positive (MDL-pruned).
    pub value: f64,
}

impl Span {
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// One division: which span, where, and the information it added (nats, net of its cost).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Step {
    pub span: usize,
    pub at: usize,
    pub gain: f64,
}

/// The whole history of a story's division.
#[derive(Debug, Clone)]
pub struct Cut {
    pub ids: Vec<u32>,
    /// Whether each token is a mark (not a word): a cut after one is cheaper to say.
    pub marks: Vec<bool>,
    pub spans: Vec<Span>,
    pub steps: Vec<Step>,
}

impl Cut {
    /// Divide `ids` (word ids `< vocab`; `marks[i]` when token `i` is not a
    /// word) completely; `prior(id)` is the distribution the whole story is read against.
    pub fn divide(ids: Vec<u32>, marks: Vec<bool>, vocab: usize, prior: &dyn Fn(u32) -> f64) -> Self {
        let mut cut = Cut { ids, marks, spans: Vec::new(), steps: Vec::new() };
        let n = cut.ids.len();
        cut.push(0, n, None, prior);
        let (mut seen, mut touched) = (vec![0u32; vocab], Vec::new());
        // Build: every span divided at its best cut, down to the word.
        let mut i = 0;
        while i < cut.spans.len() {
            if let Some((g, at)) = cut.best_division(i, prior, &mut seen, &mut touched) {
                let (s, e) = (cut.spans[i].start, cut.spans[i].end);
                let a = cut.push(s, at, Some(i), prior);
                let b = cut.push(at, e, Some(i), prior);
                cut.spans[i].division = Some((at, g));
                cut.spans[i].children = Some((a, b));
            }
            i += 1;
        }
        // Prune: children come after their parent, so walk backwards.
        for i in (0..cut.spans.len()).rev() {
            if let (Some((_, g)), Some((a, b))) = (cut.spans[i].division, cut.spans[i].children) {
                cut.spans[i].value = (g + cut.spans[a].value + cut.spans[b].value).max(0.0);
            }
        }
        // Order: the pruned tree first, most unlocked first; then the largest unit first.
        let mut frontier = vec![0usize];
        while let Some((pos, _)) = frontier.iter().enumerate().filter(|(_, s)| cut.spans[**s].division.is_some()).max_by(|(_, x), (_, y)| cut.priority(**x).partial_cmp(&cut.priority(**y)).unwrap_or(std::cmp::Ordering::Equal).then(cut.spans[**y].start.cmp(&cut.spans[**x].start))) {
            let s = frontier.swap_remove(pos);
            let step = cut.steps.len() + 1;
            let ((at, gain), (a, b)) = (cut.spans[s].division.unwrap_or_default(), cut.spans[s].children.unwrap_or_default());
            cut.steps.push(Step { span: s, at, gain });
            cut.spans[s].divided = Some(step);
            cut.spans[a].born = step;
            cut.spans[b].born = step;
            frontier.extend([a, b]);
        }
        cut
    }

    /// Which division to make next: `(1, unlocked)` inside the pruned tree, `(0, length)` below it.
    fn priority(&self, s: usize) -> (u8, f64) {
        let sp = &self.spans[s];
        match (sp.division, sp.children) {
            (Some((_, g)), Some((a, b))) if g + self.spans[a].value + self.spans[b].value > 0.0 => (1, g + self.spans[a].value + self.spans[b].value),
            _ => (0, sp.len() as f64),
        }
    }

    fn push(&mut self, a: usize, b: usize, parent: Option<usize>, prior: &dyn Fn(u32) -> f64) -> usize {
        let mut c = HashMap::new();
        for id in &self.ids[a..b] {
            *c.entry(*id).or_default() += 1;
        }
        let share = match parent {
            Some(p) => {
                let above = &self.spans[p].share;
                share_of(&c, b - a, &|id| above[&id])
            }
            None => share_of(&c, b - a, prior),
        };
        self.spans.push(Span { start: a, end: b, parent, born: 0, divided: None, counts: c, share, division: None, children: None, value: 0.0 });
        self.spans.len() - 1
    }

    /// How many units the story's information supports: the pruned tree's leaves.
    pub fn supported(&self) -> usize {
        fn leaves(c: &Cut, s: usize) -> usize {
            match c.spans[s].children {
                Some((a, b)) if c.spans[s].value > 0.0 => leaves(c, a) + leaves(c, b),
                _ => 1,
            }
        }
        leaves(self, 0)
    }

    /// The share of `id` in span `s` (the prior above the root).
    pub fn share(&self, s: Option<usize>, id: u32, prior: &dyn Fn(u32) -> f64) -> f64 {
        match s {
            Some(i) => self.spans[i].share.get(&id).copied().unwrap_or_else(|| prior(id)),
            None => prior(id),
        }
    }

    /// The spans the story is divided into once it has `units` of them.
    pub fn units_at(&self, units: usize) -> Vec<usize> {
        let step = units.saturating_sub(1);
        let mut v: Vec<usize> = (0..self.spans.len()).filter(|i| self.spans[*i].born <= step && self.spans[*i].divided.is_none_or(|d| d > step)).collect();
        v.sort_by_key(|i| self.spans[*i].start);
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_division_is_made_where_it_shortens_the_story() {
        // Two halves that share no word: the first cut falls between them, and pays for itself.
        let ids: Vec<u32> = [1, 2, 1, 2, 1, 2, 1, 2, 3, 4, 3, 4, 3, 4, 3, 4].to_vec();
        let cut = Cut::divide(ids, vec![false; 16], 5, &|_| 0.1);
        assert_eq!(cut.steps[0].at, 8, "{:?}", &cut.steps[..2]);
        assert!(cut.steps[0].gain > 0.0);
        assert_eq!(cut.steps.len(), 15, "divided until every token stands alone");
        assert_eq!(cut.supported(), 2, "the information supports the two halves, not the tokens");
        assert_eq!(cut.units_at(2).iter().map(|i| (cut.spans[*i].start, cut.spans[*i].end)).collect::<Vec<_>>(), [(0, 8), (8, 16)]);
        // Words that never repeat have nothing to say by being divided: no gain.
        let flat = Cut::divide((0..12).collect(), vec![false; 12], 12, &|_| 0.05);
        assert!(flat.steps[0].gain <= 0.0, "{:?}", flat.steps[0]);
        assert_eq!(flat.supported(), 1);
    }
}
