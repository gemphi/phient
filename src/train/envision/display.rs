#[allow(unused_imports)]
use super::*;

impl fmt::Display for Vision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "  [envision] {}", self.text)
    }
}

impl GapLedger {
    pub fn new() -> Self {
        Self::default()
    }

    pub(super) fn now_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    /// Notes that an unknown word was encountered.
    pub fn encounter(&mut self, word: &str) {
        *self.encountered.entry(word.to_string()).or_insert(0) += 1;
        self.first_seen.entry(word.to_string()).or_insert_with(Self::now_ms);
    }

    /// Whether it is still worth asking the user about this word.
    pub fn should_ask(&self, word: &str) -> bool {
        !self.resolved.contains(word) && *self.asked.get(word).unwrap_or(&0) < MAX_ASKS
    }

    pub fn mark_asked(&mut self, word: &str) {
        *self.asked.entry(word.to_string()).or_insert(0) += 1;
    }

    pub fn mark_resolved(&mut self, word: &str) {
        self.resolved.insert(word.to_string());
    }

    pub fn unresolved_count(&self) -> usize {
        self.encountered.keys().filter(|w| !self.resolved.contains(*w)).count()
    }

    /// Unresolved gaps, most frequently encountered first — a learning agenda
    /// rather than a per-turn reflex.
    pub fn unresolved_ranked(&self) -> Vec<(String, u32)> {
        let mut v: Vec<(String, u32)> = self
            .encountered
            .iter()
            .filter(|(w, _)| !self.resolved.contains(*w))
            .map(|(w, c)| (w.clone(), *c))
            .collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        v
    }
}
