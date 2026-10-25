//! Casting over the kit (`phiecs::kit::Kit`): the kit's rows counted into
//! [`Tally`] and [`ScopeTree`] so words can be cast into the body, the
//! tracking and the device. This half lives up here because counting and
//! casting are the measurement floor's; the kit itself — the rows and the
//! world — lives beside the one intermediate, in phiecs.

use super::{ScopeTree, Tally, SAYS};
use phiecs::kit::Kit;

pub trait Casting {
    /// The counts and the scope tree over everything told.
    fn knowing(&self) -> (Tally, ScopeTree);
    /// Cast `words` into the tree: the scopes drawn, best first, with the
    /// things there.
    fn cast(&self, words: &[String], beam: usize) -> Vec<(Vec<String>, Vec<String>)>;
}

impl Casting for Kit {
    fn knowing(&self) -> (Tally, ScopeTree) {
        let mut counts = Tally::new(&["has-word", "within"]);
        let mut tree = ScopeTree::default();
        counts.learn(&self.rows);
        tree.add_rows(&self.rows);
        (counts, tree)
    }

    fn cast(&self, words: &[String], beam: usize) -> Vec<(Vec<String>, Vec<String>)> {
        let (counts, tree) = self.knowing();
        tree.cast(&counts, words, f64::MIN, beam).into_iter().map(|c| { let leaf = c.path.last().cloned().unwrap_or_default(); (c.path, self.under(&leaf)) }).collect()
    }
}

/// The words a scope says, pairs left out.
pub fn said_by(counts: &Tally, scope: &str) -> Vec<String> {
    counts.said(Some(scope), SAYS).into_iter().filter(|(w, _)| !w.starts_with("pair:")).map(|(w, _)| w).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prose::tokens;

    #[test]
    fn the_kit_is_one_tree_the_body_is_drawn_from() {
        let kit = Kit::new();
        assert!(kit.rows.len() > 10_000, "{}", kit.rows.len());
        let (_, tree) = kit.knowing();
        assert_eq!(tree.roots(), ["scope:body", "scope:track", "scope:device"]);
        assert_eq!(kit.of_kind("shape").len(), 52);
        assert_eq!(kit.words("joint:left_elbow").as_deref(), Some("left elbow"));
        let ask = |q: &str| kit.cast(&tokens(q), 3).remove(0);
        // "left elbow" alone is a tie between the pose's and the skeleton's — both have one; the module's word settles it.
        assert_eq!(ask("pose left elbow").1, ["joint:left_elbow"]);
        assert_eq!(ask("skeleton left elbow").1, ["skeleton_joint:left_elbow"]);
        assert!(ask("mouth smile").1[0].starts_with("shape:mouth_smile_"), "either side");
        // "thumb tip" alone is a tie: the hand has one, and so does the skeleton's left hand and right hand.
        // A word only one of them says settles it; `hand thumb tip` goes to the skeleton's `left hand thumb tip`,
        // which says all three words itself (measured, docs/141).
        assert_eq!(ask("thumb mp").1, ["hand_joint:thumb_mp"]);
        assert_eq!(ask("index finger knuckle").1, ["hand_skeleton_joint:index_finger_knuckle"]);
        assert_eq!(ask("left hand thumb tip").1, ["skeleton_joint:left_hand_thumb_tip"]);
        // `provider class` is a key and a personality's field, both rightly; the two are the first two casts.
        let two: Vec<String> = kit.cast(&tokens("provider class"), 3).into_iter().take(2).flat_map(|c| c.1).collect();
        assert!(two.contains(&"key:provider_class".to_string()) && two.contains(&"personality.provider_class".to_string()), "{two:?}");
        assert_eq!(ask("device tree").1, ["plane:device_tree"]);
        let world = kit.world();
        assert!(world.word_count() > 1000);
    }
}
