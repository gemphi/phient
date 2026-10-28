#[allow(unused_imports)]
use super::*;

impl Role {
    pub const ALL: [Role; 6] = [
        Role::Genus,
        Role::Function,
        Role::Form,
        Role::Instance,
        Role::Context,
        Role::Property,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Role::Genus => "genus",
            Role::Function => "function",
            Role::Form => "form",
            Role::Instance => "instance",
            Role::Context => "context",
            Role::Property => "property",
        }
    }

    /// The role's rotation, one angle per channel.
    ///
    /// Derived from the role's name by the same hash the lexicon uses, so it is
    /// the same rotation in every process without being persisted, and two
    /// roles are near-orthogonal by construction. **Shared** is the whole point:
    /// a positional rotation gives the same concept a different angle in every
    /// context, and superposing scattered copies cancels.
    pub fn rotation(self) -> [f64; PHASE_CHANNELS] {
        let mut out = [0.0f64; PHASE_CHANNELS];
        for (k, o) in out.iter_mut().enumerate() {
            let h = fnv1a(&format!("role:{}:{}", self.name(), k));
            *o = (h % 100_000) as f64 / 100_000.0 * TWO_PI;
        }
        out
    }
}

impl RuleSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    pub fn add(&mut self, head: &str, role: Role, filler: &str) {
        if head == filler {
            return;
        }
        let i = self.rules.len();
        self.rules.push(Rule {
            head: head.to_string(),
            role,
            filler: filler.to_string(),
        });
        self.by_head.entry(head.to_string()).or_default().push(i);
    }

    pub fn rules_for(&self, head: &str) -> Vec<&Rule> {
        self.by_head
            .get(head)
            .map(|ix| ix.iter().map(|i| &self.rules[*i]).collect())
            .unwrap_or_default()
    }

    pub fn all(&self) -> &[Rule] {
        &self.rules
    }

    /// Tally per role, for reporting what an extractor actually found.
    pub fn role_counts(&self) -> Vec<(Role, usize)> {
        Role::ALL
            .iter()
            .map(|r| (*r, self.rules.iter().filter(|x| x.role == *r).count()))
            .collect()
    }
}
